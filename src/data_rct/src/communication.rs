use std::error::Error;
use std::io::{Read, Write};
use prost_stream::Stream;
use rand_core::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey};
use protocol::communication::{EncryptionRequest, EncryptionResponse, TransferRequest};
use crate::connection_request::ConnectionRequest;
use crate::encryption::{EncryptedStream, generate_iv};
use crate::stream::Close;

pub async fn initiate_sender_communication<T>(mut stream: T) -> Result<EncryptedStream<T>, Box<dyn Error>> where T: Read + Write + Close {
    let secret = EphemeralSecret::random_from_rng(OsRng);
    let public_key = PublicKey::from(&secret);
    let encryption_request = EncryptionRequest {
        public_key: public_key.as_bytes().to_vec()
    };

    println!("initiating prost stream");
    let mut prost_stream = Stream::new(&mut stream);
    let _ = prost_stream.send(&encryption_request);
    println!("sent request");

    println!("receiving response");
    let encryption_response: EncryptionResponse = match prost_stream.recv::<EncryptionResponse>() {
        Ok(message) => message,
        Err(error) => return Err(Box::new(error))
    };
    println!("received response");

    let public_key: [u8; 32] = encryption_response.public_key.try_into().expect("Vec length is not 32");
    let foreign_public_key = PublicKey::from(public_key);

    let shared_secret = secret.diffie_hellman(&foreign_public_key);

    let iv: [u8; 24] = encryption_response.iv.try_into().expect("Vec length is not 24");

    println!("encrypting stream");
    let encrypted_stream = EncryptedStream::new(shared_secret.to_bytes(), iv, stream);
    println!("encrypted stream");

    return Ok(encrypted_stream);
}

pub fn initiate_receiver_communication<T>(mut stream: T, file_storage: String) -> Result<ConnectionRequest, Box<dyn Error>> where T: Read + Write + Close + 'static {
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
    let mut encrypted_stream = EncryptedStream::new(shared_secret.to_bytes(), iv, stream);

    let mut prost_stream = Stream::new(&mut encrypted_stream);
    let transfer_request = match prost_stream.recv::<TransferRequest>() {
        Ok(message) => message,
        Err(error) => {
            println!("Error {:}", error);
            return Err(error.into());
        }
    };

    println!("Received Transfer Request from {:}", transfer_request.clone().device.unwrap().name);

    let connection_request = ConnectionRequest::new(
        transfer_request,
        Box::new(encrypted_stream),
        file_storage.clone()
    );

    return Ok(connection_request);
}
