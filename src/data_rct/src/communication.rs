use std::error::Error;
use std::fmt::Debug;
use std::io::{Read, Write};
use std::sync::{RwLock};
use bytes::{BufMut, BytesMut};
use prost_stream::Stream;
use rand_core::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey};
use protocol::communication::{EncryptionRequest, EncryptionResponse};
use protocol::prost::Message;
use crate::encryption::{EncryptedStream, generate_iv};


pub trait NativeStreamDelegate: Send + Sync + Debug {
    // fn read(&self, buffer_length: u64) -> Vec<u8>;
    fn write(&self, data: Vec<u8>) -> u64;
    fn close(&self);
}

pub struct NativeStream {
    buffer: RwLock<BytesMut>,
    delegate: Option<Box<dyn NativeStreamDelegate>>
}

trait StreamEncode {
    fn encode_stream<T>(reader: &mut T) where T: Read;
}

impl StreamEncode for dyn Message {
    fn encode_stream<T>(reader: &mut T) where T: Read {
        let mut buffer = [0; 4];
        let _ = reader.read_exact(&mut buffer);
    }
}

impl NativeStream {
    pub fn new(delegate: Box<dyn NativeStreamDelegate>) -> Self {
        Self {
            buffer: RwLock::new(BytesMut::new()),
            delegate: Some(delegate)
        }
    }

    pub fn fill_buffer(&self, data: Vec<u8>) {
        self.buffer.write().unwrap().put(data.as_slice());
    }

    pub fn get_buffer(&self) -> BytesMut {
        return self.buffer.write().unwrap().clone();
    }

    pub fn write(&self, data: Vec<u8>) {
        if let Some(delegate) = &self.delegate {
            delegate.write(data);
        } else {
            panic!("NativeStream delegate is not set while trying to write data!");
        }
    }

    pub fn close(&self) {
        if let Some(delegate) = &self.delegate {
            delegate.close();
        } else {
            panic!("NativeStream delegate is not set while trying to close the stream!");
        }
    }
}

pub struct SenderConnection {
}

impl SenderConnection {
    pub async fn initiate_sender<T>(mut stream: T) -> Result<EncryptedStream<T>, Box<dyn Error>> where T: Read + Write {
        let secret = EphemeralSecret::random_from_rng(OsRng);
        let public_key = PublicKey::from(&secret);
        let encryption_request = EncryptionRequest {
            public_key: public_key.as_bytes().to_vec()
        };

        println!("initiating prost stream");
        let mut prost_stream = Stream::new(&mut stream);
        let _ = prost_stream.send(&encryption_request);
        println!("sent request");

        println!("receiving request");
        let encryption_response: EncryptionResponse = match prost_stream.recv::<EncryptionResponse>() {
            Ok(message) => message,
            Err(error) => return Err(Box::new(error))
        };
        println!("received request");

        let public_key: [u8; 32] = encryption_response.public_key.try_into().expect("Vec length is not 32");
        let foreign_public_key = PublicKey::from(public_key);

        let shared_secret = secret.diffie_hellman(&foreign_public_key);

        let iv: [u8; 24] = encryption_response.iv.try_into().expect("Vec length is not 24");

        println!("encrypting stream");
        let encrypted_stream = EncryptedStream::new(shared_secret.to_bytes(), iv, stream);
        println!("encrypted stream");

        return Ok(encrypted_stream);
    }
}

pub struct ReceiverConnection {
}

impl ReceiverConnection {
    pub fn initiate_receiver<T>(mut stream: T) -> Result<EncryptedStream<T>, Box<dyn Error>> where T: Read + Write {
        let secret = EphemeralSecret::random_from_rng(OsRng);
        let public_key = PublicKey::from(&secret);

        let iv = generate_iv();

        println!("initiating prost stream");
        let mut prost_stream = Stream::new(&mut stream);

        println!("Receiving request");
        let encryption_request = match prost_stream.recv::<EncryptionRequest>() {
            Ok(message) => message,
            Err(error) => return Err(Box::new(error))
        };

        println!("Sending response");
        let _ = prost_stream.send(
            &EncryptionResponse {
                public_key: public_key.as_bytes().to_vec(),
                iv: iv.to_vec()
            }
        );

        let public_key: [u8; 32] = encryption_request.public_key.try_into().expect("Vec length is not 32");
        let foreign_public_key = PublicKey::from(public_key);

        let shared_secret = secret.diffie_hellman(&foreign_public_key);

        println!("Encrypting stream");
        let encrypted_stream = EncryptedStream::new(shared_secret.to_bytes(), iv, stream);

        return Ok(encrypted_stream);
    }
}
