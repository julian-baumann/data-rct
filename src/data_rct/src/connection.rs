use std::fmt::Debug;
use std::io::{Read, Write};
use std::io::ErrorKind::WouldBlock;
use std::sync::{RwLock};
use std::thread;
use std::time::Duration;
use bytes::{BufMut, BytesMut};
use rand_core::OsRng;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use x25519_dalek::{EphemeralSecret, PublicKey};
use protocol::communication::{EncryptionRequest, EncryptionResponse};
use protocol::prost::encoding::decode_varint;
use protocol::prost::Message;
use crate::encryption::generate_iv;

pub trait NativeStreamDelegate: Send + Sync + Debug {
    // fn read(&self, buffer_length: u64) -> Vec<u8>;
    fn write(&self, data: Vec<u8>) -> u64;
    // fn flush(&self);
    fn close(&self);
}

// impl Read for dyn NativeStreamDelegate {
//     fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
//         let length = buf.len();
//         let mut writer = BufWriter::new(buf);
//
//         let data = NativeStreamDelegate::read(self, length as u64);
//         let result = writer.write(data.as_slice());
//
//         return result;
//     }
// }
//
// impl Write for dyn NativeStreamDelegate {
//     fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
//         let result = NativeStreamDelegate::write(self, buf.to_vec());
//         return Ok(result as usize);
//     }
//
//     fn flush(&mut self) -> std::io::Result<()> {
//         NativeStreamDelegate::flush(self);
//
//         return Ok(());
//     }
// }

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

// // !! DO NOT USE!! Only for swift to extend the internal read buffer.
// pub unsafe extern "C" fn extend_native_stream(object: *mut NativeStream, pointer: *mut u8, size: usize) {
//     println!("Extending internal buffer");
//     let slice = std::slice::from_raw_parts(pointer, size);
//     let vec = slice.to_vec();
//     (*object).buffer.extend(vec);
//     println!("Extended internal buffer");
// }

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

pub struct Connection {
}

impl Connection {
    pub async fn initiate_sender<T>(stream: &mut T) where T: Read + Write {
        let secret = EphemeralSecret::random_from_rng(OsRng);
        let public_key = PublicKey::from(&secret);
        let encryption_request = EncryptionRequest {
            public_key: public_key.as_bytes().to_vec()
        };

        println!("Writing encryption request with key: {:?}", &encryption_request.public_key);
        println!("Writing encryption request with length: {:?}", &encryption_request.encoded_len());
        let message = encryption_request.encode_length_delimited_to_vec();
        let _ = stream.write(message.as_slice());
        let _ = stream.flush();
        println!("Done writing");

        let mut buffer = BytesMut::zeroed(1);

        Connection::fill_buffer(&mut buffer, stream);

        let encryption_response = EncryptionResponse::decode_length_delimited(&mut buffer);

        if let Err(decode_error) = encryption_response {
            println!("Error: {:?}", decode_error);
            return;
        };

        let Ok(encryption_response) = encryption_response else {
            return;
        };

        println!("Received encryption request. Key: {0:?}, IV: {1:?}", encryption_response.public_key, encryption_response.iv);
    }

    fn read_length_delimited<T>(buffer: &mut BytesMut, stream: &mut T) where T : Read + Write {
        let mut length_delimiter_buf = [0u8; 2];
        stream.read_exact(&mut length_delimiter_buf).expect("Failed to read_exact");

        let mut length_buf = [0u8; 4];
        stream.read_exact(&mut length_buf).expect("Failed to read_exact");
        println!("length buf {:?}", length_buf);

        let msg_length = i32::from_le_bytes(length_buf) as usize;

        println!("Message length: {:}", msg_length);

        println!("Resizing");
        buffer.resize(msg_length, 0);
        println!("Reading");
        stream.read_exact(buffer).expect("Failed to fill buffer");
        println!("Read everything");
    }

    fn fill_buffer<T>(buffer: &mut BytesMut, stream: &mut T) where T : Read + Write {
        let result = stream.read(buffer).expect("Failed to fill buffer");
        println!("Read {:} into buffer", result);
    }

    pub fn initiate_receiver<T>(mut stream: T) where T: Read + Write {
        let secret = EphemeralSecret::random_from_rng(OsRng);
        let public_key = PublicKey::from(&secret);

        let iv = generate_iv();

        let mut buffer = BytesMut::zeroed(1);
        Connection::fill_buffer(&mut buffer, &mut stream);

        println!("Prepared buffer");

        println!("Reading EncryptionRequest...");
        let encryption_request = EncryptionRequest::decode_length_delimited(&mut buffer);
        println!("Received encryption request: {:?}", encryption_request.unwrap().public_key);

        println!("Writing response...");
        let _ = stream.write(
            EncryptionResponse {
                public_key: public_key.as_bytes().to_vec(),
                iv: iv.to_vec()
            }.encode_length_delimited_to_vec().as_slice()
        );

        let _ = stream.flush();

        println!("Written response");
    }
}
