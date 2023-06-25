use std::{io};
use std::io::{Error, Read, Write};
use std::io::ErrorKind::Other;
use std::iter::repeat;
use rand_core::{OsRng};
use chacha20::{XChaCha20};
use chacha20::cipher::{KeyIvInit, StreamCipher};
use crate::stream::{Stream, StreamRead, StreamWrite};

pub fn generate_key() -> [u8; 32] {
    let key = XChaCha20::generate_key(&mut OsRng);

    return key.into();
}

pub fn generate_iv() -> [u8; 24] {
    let nonce = XChaCha20::generate_iv(&mut OsRng);

    return nonce.into();
}

pub struct EncryptedStream {
    pub cipher: XChaCha20,
    pub raw_stream: Box<dyn Stream>
}

impl StreamRead for EncryptedStream {}
impl StreamWrite for EncryptedStream {}

impl<'a> EncryptedStream {
    pub fn new(key: [u8; 32], iv: [u8; 24], stream: Box<dyn Stream>) -> Self {
        let cipher = XChaCha20::new(&key.into(), &iv.into());

        Self {
            cipher,
            raw_stream: stream
        }
    }
}

impl<'a> Read for EncryptedStream {
    fn read(&mut self, read_buffer: &mut [u8]) -> io::Result<usize> {
        let mut buffer: Vec<u8> = repeat(0).take(read_buffer.len()).collect();
        let read_bytes = self.raw_stream.read(&mut buffer);

        if let Ok(read_bytes) = read_bytes {
            if read_bytes == 0 {
                return Ok(read_bytes);
            }

            let sized_buffer = &buffer[..read_bytes];

            let decrypted_message_part = self.cipher.apply_keystream_b2b(sized_buffer, &mut read_buffer[..read_bytes]);

            return if let Err(error) = decrypted_message_part {
                Err(Error::new(Other, format!("{}", error.to_string())))
            } else {
                Ok(read_bytes)
            }
        }

        return Ok(0);
    }
}

impl Write for EncryptedStream {
    fn write(&mut self, write_buffer: &[u8]) -> io::Result<usize> {
        let mut buffer: Vec<u8> = repeat(0).take(write_buffer.len()).collect();
        let ciphertext = self.cipher.apply_keystream_b2b(write_buffer, &mut buffer);

        if let Ok(()) = ciphertext {
            return self.raw_stream.write(&buffer);
        }

        return Ok(0);
    }

    fn flush(&mut self) -> io::Result<()> {
        self.raw_stream.flush()
    }
}
