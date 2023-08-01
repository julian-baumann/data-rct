use std::error::Error;
use anyhow::{Result};
use std::io::{Read, Write};
use crate::discovery::DeviceInfo;
use crate::transmission::tcp::{TcpTransmissionClient, TcpTransmissionListener};
use std::net::{ToSocketAddrs};
use std::sync::Arc;
use crate::PROTOCOL_VERSION;
use uuid::Uuid;
use thiserror::Error;
use x25519_dalek::{EphemeralSecret, PublicKey};
use rand_core::OsRng;
use crate::encryption::{EncryptedStream, generate_iv};
use crate::stream::{check_result, Stream, IncomingErrors, ConnectErrors, StreamRead, StreamWrite};

mod tcp;

const PUBLIC_KEY_SIZE: usize = 32;
const UUID_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 24;
const ACCEPT_TRANSMISSION: u8 = 1;
const DENY_TRANSMISSION: u8 = 0;

trait DataTransmission {
    fn new() -> Result<Self, Box<dyn Error>> where Self: Sized;
    fn accept(&self) -> Option<Box<dyn Stream>>;
}

pub enum TransmissionMessageTunnel {
    ReceivedTransfer(Box<dyn Stream>)
}

pub struct TransmissionRequest {
    pub uuid: String,
    pub sender_id: String,
    pub sender_name: String,
    pub data_stream: Arc<EncryptedStream>
}

impl TransmissionRequest {
    pub fn accept(&self) -> Result<EncryptedStream> {
        self.data_stream.write_immutable(&[ACCEPT_TRANSMISSION])?;

        return Ok(
            self.data_stream.as_ref().clone()
        );
    }

    pub fn deny(&self) -> Result<()> {
        self.data_stream.write_immutable(&[DENY_TRANSMISSION])?;

        return Ok(());
    }
}


#[derive(Error, Debug)]
pub enum TransmissionSetupError {
    #[error("Unable to start TCP server.")]
    UnableToStartTcpServer
}

pub struct Transmission {
    pub device_info: DeviceInfo,
    tcp_transmission: TcpTransmissionListener
}

impl Transmission {
    pub fn new(device_info: DeviceInfo) -> Result<Self, TransmissionSetupError> {
        let tcp_transmission = match TcpTransmissionListener::new() {
            Ok(result) => result,
            Err(_) => return Err(TransmissionSetupError::UnableToStartTcpServer)
        };


        let mut modified_device = device_info.clone();
        modified_device.port = tcp_transmission.port;

        return Ok(Transmission {
            device_info: modified_device,
            tcp_transmission
        });
    }

    pub fn get_port(&self) -> u16 {
        return self.tcp_transmission.port;
    }

    fn encrypt_stream<'b>(&'b self, my_key: EphemeralSecret, foreign_key: [u8; 32], nonce: [u8; 24], stream: Box<dyn Stream>) -> Result<EncryptedStream, Box<dyn Error>> {
        let foreign_key = PublicKey::from(foreign_key);
        let shared_key = my_key.diffie_hellman(&foreign_key);

        let encrypted_stream = EncryptedStream::new(shared_key.to_bytes(), nonce, stream);

        return Ok(encrypted_stream);
    }

    pub fn get_incoming(&self) -> Option<TransmissionRequest> {
        let request = self.get_incoming_with_errors();

        if let Some(request) = request {
            if let Ok(transmission_request) = request {
                return Some(transmission_request);
            } else if let Err(error) = request {
                eprintln!("{}", error);
            }
        }

        return None;
    }

    pub fn get_incoming_with_errors(&self) -> Option<Result<TransmissionRequest, IncomingErrors>> {
        if let Some(mut connection) = self.tcp_transmission.accept() {
            // == Version ==
            let protocol_version = connection.read_u8();

            if let Err(error) = protocol_version {
                return Some(Err(IncomingErrors::UnknownReadError(error)));
            } else if let Ok(version) = protocol_version {
                if version > PROTOCOL_VERSION {
                    return Some(Err(IncomingErrors::InvalidVersion));
                }
            }

            let uuid = match connection.read_and_get_value(UUID_LENGTH, IncomingErrors::InvalidUUID) {
                Ok(value) => value,
                Err(error) => return Some(Err(error))
            };

            let uuid = match Uuid::from_slice(uuid.as_slice()) {
                Ok(value) => value,
                Err(_) => return Some(Err(IncomingErrors::InvalidUUID))
            };

            let foreign_public_key = match connection.read_and_get_value(PUBLIC_KEY_SIZE, IncomingErrors::InvalidForeignPublicKey) {
                Ok(value) => value,
                Err(error) => return Some(Err(error))
            };

            let session_secret_key = EphemeralSecret::new(OsRng);
            let session_public_key = PublicKey::from(&session_secret_key);
            let result = connection.write(session_public_key.as_bytes());

            if let Some(error) = check_result(result, IncomingErrors::ErrorSendingPublicKey, Some(PUBLIC_KEY_SIZE)) {
                return Some(Err(error));
            }

            let nonce = match connection.read_and_get_value(NONCE_LENGTH, IncomingErrors::InvalidNonce) {
                Ok(value) => value,
                Err(error) => return Some(Err(error))
            };

            let mut encrypted_stream = match self.encrypt_stream(
                session_secret_key,
                <[u8; 32]>::try_from(foreign_public_key).unwrap(),
                <[u8; 24]>::try_from(nonce).unwrap(),
                connection) {
                Ok(value) => value,
                Err(_) => return Some(Err(IncomingErrors::EncryptionError))
            };

            let sender_id_length = match encrypted_stream.read_u8() {
                Ok(value) => value,
                Err(error) => return Some(Err(IncomingErrors::UnknownReadError(error)))
            };

            // == Sender ID ==
            let sender_id = match encrypted_stream.read_string(sender_id_length as usize, IncomingErrors::InvalidSenderId) {
                Ok(value) => value,
                Err(error) => return Some(Err(error))
            };

            let sender_name_length = match encrypted_stream.read_u8() {
                Ok(value) => value,
                Err(error) => return Some(Err(IncomingErrors::UnknownReadError(error)))
            };

            // == Sender Name ==
            let sender_name = match encrypted_stream.read_string(sender_name_length as usize, IncomingErrors::InvalidSenderName) {
                Ok(value) => value,
                Err(error) => return Some(Err(error))
            };

            return Some(Ok(TransmissionRequest {
                uuid: uuid.to_string(),
                sender_id,
                sender_name,
                data_stream: Arc::new(encrypted_stream)
            }));
        }

        return None;
    }

    pub fn open(&self, recipient: &DeviceInfo) -> Result<EncryptedStream, ConnectErrors> {
        let socket_address = (recipient.ip_address.as_str(), recipient.port).to_socket_addrs();

        let mut socket_address = match socket_address {
            Ok(address) => address,
            Err(_) => return Err(ConnectErrors::InvalidSocketAddress),
        };

        let socket_address = match socket_address.next() {
            Some(address) => address,
            None => return Err(ConnectErrors::InvalidSocketAddress),
        };

        let connection = TcpTransmissionClient::connect(socket_address);

        let connection = match connection {
            Ok(address) => address,
            Err(error) => return Err(ConnectErrors::CouldNotOpenSocket(error.to_string())),
        };

        return self.connect(Box::new(connection));
    }

    fn connect(&self, mut connection: Box<dyn Stream>) -> Result<EncryptedStream, ConnectErrors>  {
        let transfer_id = Uuid::new_v4();

        // Send core header information
        match connection.write_stream(&[PROTOCOL_VERSION]) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };
        match connection.write_stream(transfer_id.as_bytes()) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };

        // Diffie-hellman key exchange - send my key, read foreign key -> generate combined key for session encryption
        let session_secret_key = EphemeralSecret::new(OsRng);
        let session_public_key = PublicKey::from(&session_secret_key);

        match connection.write_stream(session_public_key.as_bytes()) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };

        let mut foreign_public_key_buffer: [u8; PUBLIC_KEY_SIZE] = [0; PUBLIC_KEY_SIZE];

        let bytes_read = connection.read(&mut foreign_public_key_buffer);

        if let Ok(bytes_read) = bytes_read {
            if bytes_read != PUBLIC_KEY_SIZE {
                return Err(ConnectErrors::InvalidForeignPublicKey("Wrong size".to_string()));
            }
        } else if let Err(error) = bytes_read {
            return Err(ConnectErrors::InvalidForeignPublicKey(error.to_string()));
        }

        // Generate random nonce for this session
        let nonce = generate_iv();
        match connection.write_stream(nonce.as_slice()) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };

        let mut encrypted_stream = match self.encrypt_stream(session_secret_key, foreign_public_key_buffer, nonce, connection) {
            Ok(value) => value,
            Err(error) => return Err(ConnectErrors::EncryptionError(error.to_string()))
        };

        // Send my id.
        match encrypted_stream.write_stream(&[self.device_info.id.len() as u8]) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };
        match encrypted_stream.write_stream(self.device_info.id.as_bytes()) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };

        // Since the sender name can vary in length, send the length beforehand, so the other end knows what to expect
        let mut sender_name_in_bytes = self.device_info.name.as_bytes();
        if sender_name_in_bytes.len() > 50 {
            sender_name_in_bytes = self.device_info.name[..50].as_bytes();
        }

        match encrypted_stream.write_stream(&[sender_name_in_bytes.len() as u8]) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };
        match encrypted_stream.write_stream(sender_name_in_bytes) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };

        loop {
            let mut answer_buffer: [u8; 1] = [0];
            let answer_available = match encrypted_stream.read(&mut answer_buffer) {
                Ok(value) => value,
                Err(error) => return Err(ConnectErrors::UnknownReadError(error))
            };

            if answer_available > 0 {
                return if answer_buffer[0] == ACCEPT_TRANSMISSION {
                    Ok(encrypted_stream)
                } else {
                    Err(ConnectErrors::Rejected)
                };
            }
        }
    }
}
