use std::error::Error;
use std::io::{Read, Write};
use crate::discovery::DeviceInfo;
use crate::transmission::tcp_transmission::{TcpTransmissionClient, TcpTransmissionListener};
use std::net::{ToSocketAddrs};
use crate::PROTOCOL_VERSION;
use uuid::Uuid;
use x25519_dalek::{EphemeralSecret, PublicKey};
use rand_core::OsRng;

mod tcp_transmission;


const PUBLIC_KEY_SIZE: usize = 32;

trait DataTransmission {
    fn new() -> Result<Self, Box<dyn Error>> where Self: Sized;
    fn accept(&self) -> Option<Box<dyn Stream>>;
}

pub trait Stream: Read + Write {}

pub enum TransmissionMessageTunnel {
    ReceivedTransfer(Box<dyn Stream>)
}

pub struct Transmission<'a> {
    device_info: &'a DeviceInfo,
    tcp_transmission: TcpTransmissionListener
}

impl<'a> Transmission<'a> {
    pub fn new(device_info: &'a DeviceInfo) -> Result<Self, Box<dyn Error>> {
        return Ok(Transmission {
            device_info,
            tcp_transmission: TcpTransmissionListener::new()?
        })
    }

    pub fn accept(&self) -> Option<Box<dyn Stream>> {
        if let Some(mut stream) = self.tcp_transmission.accept() {
            // Send first identification byte
            stream.write(&[0x00]).ok()?;
            stream.write(&[0x00]).ok()?;
        }

        return None;
    }

    pub fn open(&self, recipient: &DeviceInfo) -> Result<(), Box<dyn Error>> {
        let socket_address = (recipient.ip_address.as_str(), recipient.port).to_socket_addrs()?.next();

        let socket_address = match socket_address {
            Some(address) => address,
            None => return Err("Something went wrong, while trying to get the SocketAddr")?,
        };


        let connection = TcpTransmissionClient::connect(socket_address)?;
        return self.connect(Box::new(connection), recipient);
    }

    pub fn connect(&self, mut connection: Box<dyn Stream>, recipient: &DeviceInfo) -> Result<(), Box<dyn Error>>  {
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

        // Send my id.
        connection.write(recipient.id.as_bytes())?;

        // Since the sender name can vary in length, send the length beforehand, so the other end knows what to expect
        let sender_name_in_bytes = self.device_info.name[..50].as_bytes();
        connection.write(&[sender_name_in_bytes.len() as u8])?;
        connection.write(sender_name_in_bytes)?;



        return Ok(());
    }
}