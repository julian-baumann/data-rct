use std::{io};
use std::io::{Error, Read, Write};
use std::io::ErrorKind::Other;
use std::iter::repeat;
use chacha20poly1305::{AeadCore, ChaChaPoly1305, KeyInit, XChaCha20Poly1305};
use chacha20poly1305::aead::{Aead, stream};
use chacha20poly1305::consts::U24;
use rand_core::{OsRng};
use chacha20::XChaCha20;

use crate::transmission::Stream;

pub fn generate_key() -> Vec<u8> {
    let key = XChaCha20Poly1305::generate_key(&mut OsRng);

    return key.to_vec();
}

pub fn generate_nonce() -> Vec<u8> {
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);

    return nonce.to_vec();
}

pub struct EncryptedStream<'a> {
    encryption_engine: ChaChaPoly1305<XChaCha20, U24>,
    iv: &'a [u8],
    pub raw_stream: Box<&'a mut dyn Stream>
}

impl<'a> EncryptedStream<'a> {
    pub fn new(key: &[u8], iv: &'a [u8], stream: Box<&'a mut dyn Stream>) -> Self {
        let encryption_engine = XChaCha20Poly1305::new(key.into());
        let stream_encryptor = stream::EncryptorBE32::from_aead(encryption_engine, iv.as_ref().into());

        Self {
            encryption_engine,
            iv,
            raw_stream: stream
        }
    }
}

impl<'a> Read for EncryptedStream<'a> {
    fn read(&mut self, read_buffer: &mut [u8]) -> io::Result<usize> {
        let mut buffer: Vec<u8> = repeat(0).take(read_buffer.len()).collect();
        let read_bytes = self.raw_stream.read(&mut buffer);

        if let Ok(read_bytes) = read_bytes {
            if read_bytes <= 0 {
                return Ok(read_bytes);
            }

            let decrypted = self.encryption_engine.decrypt(self.iv.into(), buffer[..read_bytes].as_ref());

            if let Err(error) = decrypted {
                return Err(Error::new(Other, format!("{:?}", error)));
            }

            if let Ok(decrypted) = decrypted {
                read_buffer[..decrypted.len()].copy_from_slice(decrypted.as_slice());

                return Ok(decrypted.len())
            }

            return Ok(0);
        }

        return Ok(0);
    }
}

impl<'a> Write for EncryptedStream<'a> {
    fn write(&mut self, write_buffer: &[u8]) -> io::Result<usize> {

        let encrypted = self.encryption_engine.encrypt(self.iv.into(), write_buffer.as_ref());

        if let Ok(encrypted) = encrypted {
            return self.raw_stream.write(&encrypted);
        }

        return Ok(0);
    }

    fn flush(&mut self) -> io::Result<()> {
        self.raw_stream.flush()
    }
}