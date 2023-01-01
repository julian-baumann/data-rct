use std::{io};
use std::io::{Error, ErrorKind, Read, Write};
use std::io::ErrorKind::Other;
use std::iter::repeat;
use chacha20poly1305::{aead, AeadCore, ChaChaPoly1305, KeyInit, XChaCha20Poly1305};
use chacha20poly1305::aead::stream::{Decryptor, Encryptor, StreamBE32};
use chacha20poly1305::consts::U24;
use rand_core::{OsRng};
use chacha20::XChaCha20;
use chacha20poly1305::aead::stream;

use crate::transmission::Stream;

pub fn generate_key() -> Vec<u8> {
    let key = XChaCha20Poly1305::generate_key(&mut OsRng);

    return key.to_vec();
}

pub fn generate_nonce() -> Vec<u8> {
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);

    return nonce[..19].to_vec();
}

pub struct EncryptedStream<'a> {
    encryptor: Encryptor<ChaChaPoly1305<XChaCha20, U24>, StreamBE32<ChaChaPoly1305<XChaCha20, U24>>>,
    decryptor: Decryptor<ChaChaPoly1305<XChaCha20, U24>, StreamBE32<ChaChaPoly1305<XChaCha20, U24>>>,
    pub raw_stream: Box<&'a mut dyn Stream>
}

impl<'a> EncryptedStream<'a> {
    pub fn new(key: &[u8], nonce: &'a [u8], stream: Box<&'a mut dyn Stream>) -> Self {
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
            Err(error) => Err(Error::new(Other, format!("An error occurred while trying to encrypt stream buffer {}", error)))
        }
    }

    pub fn read_last(mut self, mut read_buffer: &mut [u8]) -> io::Result<usize> {
        let length = if read_buffer.len() > 0 { read_buffer.len() + 20 } else { 120 };
        let mut buffer: Vec<u8> = repeat(0).take(length).collect();
        let read_bytes = self.raw_stream.read(&mut buffer);

        if let Ok(read_bytes) = read_bytes {
            if read_bytes <= 0 {
                return Ok(read_bytes);
            }

            let sized_buffer = &buffer[..read_bytes];

            let decrypted_message_part = self.decryptor.decrypt_last(sized_buffer);

            if let Ok(decrypted_message_part) = decrypted_message_part {
                let result = read_buffer.write(decrypted_message_part.as_slice());

                if let Err(error) = result {
                    return Err(error);
                }

                return Ok(read_bytes);
            } else if let Err(error) = decrypted_message_part {
                return Err(Error::new(Other, format!("An error occurred while trying to decrypt stream buffer {error}")));
            }
        }

        return Ok(0);
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

            let sized_buffer = &buffer[..read_bytes];

            let decrypted_message_part = self.decryptor.decrypt_next(sized_buffer);

            if let Err(error) = decrypted_message_part {
                return Err(Error::new(Other, format!("{}", (error as aead::Error).to_string())));
            }

            if let Ok(decrypted) = decrypted_message_part {
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

        let ciphertext = self.encryptor.encrypt_next(write_buffer);

        if let Ok(encrypted) = ciphertext {
            return self.raw_stream.write(&encrypted);
        }

        return Ok(0);
    }

    fn flush(&mut self) -> io::Result<()> {
        self.raw_stream.flush()
    }
}