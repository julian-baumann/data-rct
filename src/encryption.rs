// -- Based on https://kerkour.com/rust-file-encryption --

use std::io;
use std::io::{Error, ErrorKind, Read, Write};
use anyhow::anyhow;
use chacha20::cipher::KeyIvInit;
use chacha20::XChaCha20;
use chacha20poly1305::{aead::{stream}, ChaChaPoly1305, KeyInit, XChaCha20Poly1305};
use chacha20poly1305::aead::stream::{Decryptor, Encryptor, StreamBE32};
use chacha20poly1305::consts::U24;
use rand_core::{OsRng, RngCore};
use crate::transmission::Stream;

pub fn generate_nonce() -> [u8; 19] {
    let mut nonce = [0u8; 19];
    OsRng.fill_bytes(&mut nonce);

    return nonce;
}

pub struct EncryptedStream<'a> {
    encryptor: Encryptor<ChaChaPoly1305<XChaCha20, U24>, StreamBE32<ChaChaPoly1305<XChaCha20, U24>>>,
    decryptor: Decryptor<ChaChaPoly1305<XChaCha20, U24>, StreamBE32<ChaChaPoly1305<XChaCha20, U24>>>,
    pub raw_stream: Box<&'a mut dyn Stream>
}

impl<'a> EncryptedStream<'a> {
    pub fn new(key: &[u8; 32], nonce: &[u8; 19], stream: Box<&'a mut dyn Stream>) -> Self {
        let encryptor = stream::EncryptorBE32::from_aead(XChaCha20Poly1305::new(key.into()), nonce.into());
        let decryptor = stream::DecryptorBE32::from_aead(XChaCha20Poly1305::new(key.into()), nonce.into());

        Self {
            encryptor,
            decryptor,
            raw_stream: stream
        }
    }

    pub fn write_last(mut self, write_buffer: &[u8]) -> io::Result<usize> {
        let ciphertext = self.encryptor.encrypt_last(write_buffer);

        return match ciphertext {
            Ok(ciphertext) => self.raw_stream.write(&ciphertext),
            Err(error) => Err(Error::new(ErrorKind::Other, format!("An error occurred while trying to encrypt stream buffer {}", error)))
        }
    }

    pub fn read_last(mut self, mut read_buffer: &mut [u8]) -> io::Result<usize> {
        // let mut buffer = Vec::new();
        let mut buffer = [0u8; 19];
        let read_bytes = self.raw_stream.read(&mut buffer);

        if let Ok(read_bytes) = read_bytes {
            if read_bytes <= 0 {
                return Ok(read_bytes);
            }

            let decrypted_message_part = self.decryptor.decrypt_last(buffer.as_slice());

            if let Ok(decrypted_message_part) = decrypted_message_part {
                let result = read_buffer.write(decrypted_message_part.as_slice());

                if let Err(error) = result {
                    return Err(error);
                }

                return Ok(read_bytes);
            } else if let Err(error) = decrypted_message_part {
                return Err(Error::new(ErrorKind::Other, format!("An error occurred while trying to decrypt stream buffer {error}")));
            }
        }

        return Ok(0);
    }
}

impl<'a> Read for EncryptedStream<'a> {
    fn read(&mut self, mut read_buffer: &mut [u8]) -> io::Result<usize> {
        // let mut buffer = Vec::new();
        let mut buffer = [0u8; 19];
        let read_bytes = self.raw_stream.read(&mut buffer);

        if let Ok(read_bytes) = read_bytes {
            if read_bytes <= 0 {
                return Ok(read_bytes);
            }

            let decrypted_message_part = self.decryptor.decrypt_next(buffer.as_slice());

            if let Ok(decrypted_message_part) = decrypted_message_part {
                let result = read_buffer.write(decrypted_message_part.as_slice());

                if let Err(error) = result {
                    return Err(error);
                }

                return Ok(read_bytes);

            } else if let Err(error) = decrypted_message_part {
                return Err(Error::new(ErrorKind::Other, format!("An error occurred while trying to decrypt stream buffer {error}")));
            }
        }

        return Ok(0);
    }
}

impl<'a> Write for EncryptedStream<'a> {
    fn write(&mut self, write_buffer: &[u8]) -> io::Result<usize> {
        let ciphertext = self.encryptor.encrypt_next(write_buffer);

        return match ciphertext {
            Ok(ciphertext) => self.raw_stream.write(&ciphertext),
            Err(error) => Err(Error::new(ErrorKind::Other, format!("An error occurred while trying to encrypt stream buffer {}", error)))
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.raw_stream.flush()
    }
}