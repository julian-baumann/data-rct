use std::error::Error;
use std::io;
use anyhow::{Result};
use std::io::{Read, Write};
use crate::discovery::DeviceInfo;
use crate::transmission::tcp::{TcpTransmissionClient, TcpTransmissionListener};
use std::net::{ToSocketAddrs};
use downcast_rs::{DowncastSync, impl_downcast};
use crate::PROTOCOL_VERSION;
use uuid::Uuid;
use x25519_dalek::{EphemeralSecret, PublicKey};
use rand_core::OsRng;
use crate::encryption::{EncryptedStream, generate_nonce};
use thiserror::Error;

mod tcp;


const PUBLIC_KEY_SIZE: usize = 32;
const UUID_LENGTH: usize = 36;
const NONCE_LENGTH: usize = 24;

trait DataTransmission {
    fn new() -> Result<Self, Box<dyn Error>> where Self: Sized;
    fn accept(&self) -> Option<Box<dyn Stream>>;
}

pub trait StreamReadExtension: Read {
    fn read_u8(&mut self) -> std::result::Result<u8, io::Error> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_and_get_value<'a>(&mut self, length: usize, error: AcceptErrors) -> Result<Vec<u8>, AcceptErrors> {
        let mut buffer = vec![0u8; length];
        let slice_buffer = buffer.as_mut_slice();
        let result = self.read(slice_buffer);

        if let Some(error) = check_result(result, error, Some(length)) {
            return Err(error);
        }

        return Ok(slice_buffer.to_owned());
    }

    fn read_string(&mut self, length: usize, error: AcceptErrors) -> Result<String, AcceptErrors> {
        let value = match self.read_and_get_value(length, error) {
            Ok(value) => value,
            Err(error) => return Err(error)
        };

        let value_as_string = match String::from_utf8(value) {
            Ok(value) => value,
            Err(_) => return Err(AcceptErrors::StringConversionError)
        };

        return Ok(value_as_string);
    }
}

pub trait Stream: StreamReadExtension + Write + DowncastSync {
}
impl_downcast!(sync Stream);

pub enum TransmissionMessageTunnel {
    ReceivedTransfer(Box<dyn Stream>)
}

pub struct Transmission {
    pub device_info: DeviceInfo,
    tcp_transmission: TcpTransmissionListener
}

#[derive(Error, Debug)]
pub enum AcceptErrors {
    #[error("Unknown reading error")]
    UnknownReadingError(io::Error),

    #[error("Error while trying to convert utf8-sequence to string")]
    StringConversionError,

    #[error("Missing protocol version")]
    MissingProtocolVersion,

    #[error("Invalid version")]
    InvalidVersion,

    #[error("Invalid uuid")]
    InvalidUUID,

    #[error("Invalid foreign public key")]
    InvalidForeignPublicKey,

    #[error("Error sending public key")]
    ErrorSendingPublicKey,

    #[error("Invalid nonce")]
    InvalidNonce,

    #[error("Encryption error")]
    EncryptionError,

    #[error("Invalid sender-id")]
    InvalidSenderId,

    #[error("Invalid sender-name")]
    InvalidSenderName
}

pub struct TransmissionRequest {
    pub uuid: String,
    pub sender_id: String,
    pub sender_name: String,
    pub data_stream: EncryptedStream
}

fn check_result(result: core::result::Result<usize, io::Error>, error: AcceptErrors, expected_length: Option<usize>) -> Option<AcceptErrors> {
    if let Err(error) = result {
        return Some(AcceptErrors::UnknownReadingError(error));
    }

    if let Some(expected_length) = expected_length {
        if let Ok(bytes_read) = result {
            if bytes_read > expected_length {
                return Some(error);
            }
        }
    }

    return None;
}

impl Transmission {
    pub fn new(device_info: DeviceInfo) -> Result<Self, Box<dyn Error>> {
        let tcp_transmission =  TcpTransmissionListener::new()?;
        let mut modified_device = device_info.clone();
        modified_device.port = tcp_transmission.port;

        return Ok(Transmission {
            device_info: modified_device,
            tcp_transmission
        })
    }

    pub fn accept(&self) -> Option<Result<TransmissionRequest, AcceptErrors>> {
        if let Some(mut connection) = self.tcp_transmission.accept() {
            // == Version ==
            let protocol_version = connection.read_u8();

            if let Err(error) = protocol_version {
                return Some(Err(AcceptErrors::UnknownReadingError(error)));
            } else if let Ok(version) = protocol_version {
                if version > PROTOCOL_VERSION {
                    return Some(Err(AcceptErrors::InvalidVersion));
                }
            }

            let uuid = match connection.read_string(UUID_LENGTH, AcceptErrors::InvalidUUID) {
                Ok(value) => value,
                Err(error) => return Some(Err(error))
            };

            let foreign_public_key = match connection.read_and_get_value(PUBLIC_KEY_SIZE, AcceptErrors::InvalidForeignPublicKey) {
                Ok(value) => value,
                Err(error) => return Some(Err(error))
            };

            let session_secret_key = EphemeralSecret::new(OsRng);
            let session_public_key = PublicKey::from(&session_secret_key);
            let result = connection.write(session_public_key.as_bytes());

            if let Some(error) = check_result(result, AcceptErrors::ErrorSendingPublicKey, Some(PUBLIC_KEY_SIZE)) {
                return Some(Err(error));
            }

            let nonce = match connection.read_and_get_value(NONCE_LENGTH, AcceptErrors::InvalidNonce) {
                Ok(value) => value,
                Err(error) => return Some(Err(error))
            };

            let mut encrypted_stream = match self.encrypt_stream(
                session_secret_key,
                <[u8; 32]>::try_from(foreign_public_key).unwrap(),
                <[u8; 24]>::try_from(nonce).unwrap(),
                connection) {
                Ok(value) => value,
                Err(_) => return Some(Err(AcceptErrors::EncryptionError))
            };

            let sender_id_length = match encrypted_stream.read_u8() {
                Ok(value) => value,
                Err(error) => return Some(Err(AcceptErrors::UnknownReadingError(error)))
            };

            // == Sender ID ==
            let sender_id = match encrypted_stream.read_string(sender_id_length as usize, AcceptErrors::InvalidSenderId) {
                Ok(value) => value,
                Err(error) => return Some(Err(error))
            };

            let sender_name_length = match encrypted_stream.read_u8() {
                Ok(value) => value,
                Err(error) => return Some(Err(AcceptErrors::UnknownReadingError(error)))
            };

            // == Sender Name ==
            let sender_name = match encrypted_stream.read_string(sender_name_length as usize, AcceptErrors::InvalidSenderName) {
                Ok(value) => value,
                Err(error) => return Some(Err(error))
            };

            return Some(Ok(TransmissionRequest {
                uuid,
                sender_id,
                sender_name,
                data_stream: encrypted_stream
            }));
        }

        return None;
    }

    pub fn open(&self, recipient: &DeviceInfo) -> Result<EncryptedStream, Box<dyn Error>> {
        let socket_address = (recipient.ip_address.as_str(), recipient.port).to_socket_addrs()?.next();

        let socket_address = match socket_address {
            Some(address) => address,
            None => return Err("Something went wrong, while trying to get the SocketAddr")?,
        };


        let connection = TcpTransmissionClient::connect(socket_address)?;
        return self.connect(Box::new(connection), recipient);
    }

    pub fn connect(&self, mut connection: Box<dyn Stream>, recipient: &DeviceInfo) -> Result<EncryptedStream, Box<dyn Error>>  {
        let transfer_id = Uuid::new_v4();

        // Send core header information
        connection.write(&[PROTOCOL_VERSION])?;
        connection.write(transfer_id.as_bytes())?;

        // Diffie-hellman key exchange - send my key, read foreign key -> generate combined key for session encryption
        let session_secret_key = EphemeralSecret::new(OsRng);
        let session_public_key = PublicKey::from(&session_secret_key);

        connection.write(session_public_key.as_bytes())?;

        let mut foreign_public_key_buffer: [u8; PUBLIC_KEY_SIZE] = [0; PUBLIC_KEY_SIZE];

        let bytes_read = connection.read(&mut foreign_public_key_buffer);

        if let Ok(bytes_read) = bytes_read {
            if bytes_read != PUBLIC_KEY_SIZE {
                return Err("Wrong size for foreign public key!")?;
            }
        } else {
            return Err("Error while trying to read foreign public key")?;
        }

        // Generate random nonce for this session
        let nonce = generate_nonce();
        connection.write(nonce.as_slice())?;

        let mut encrypted_stream = self.encrypt_stream(session_secret_key, foreign_public_key_buffer, nonce, connection)?;

        // Send my id.
        encrypted_stream.write(&[recipient.id.len() as u8])?;
        encrypted_stream.write(recipient.id.as_bytes())?;

        // Since the sender name can vary in length, send the length beforehand, so the other end knows what to expect
        let sender_name_in_bytes = self.device_info.name[..50].as_bytes();
        encrypted_stream.write(&[sender_name_in_bytes.len() as u8])?;
        encrypted_stream.write(sender_name_in_bytes)?;

        return Ok(encrypted_stream);
    }

    fn encrypt_stream<'b>(&'b self, my_key: EphemeralSecret, foreign_key: [u8; 32], nonce: [u8; 24], stream: Box<dyn Stream>) -> Result<EncryptedStream, Box<dyn Error>> {
        let foreign_key = PublicKey::from(foreign_key);
        let shared_key = my_key.diffie_hellman(&foreign_key);

        let encrypted_stream = EncryptedStream::new(shared_key.to_bytes(), nonce, stream);

        return Ok(encrypted_stream);
    }
}