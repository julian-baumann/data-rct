// -- Based on https://kerkour.com/rust-file-encryption --

use std::io;
use std::io::{Error, ErrorKind, Read, Write};
use chacha20::XChaCha20;

use chacha20poly1305::{aead::{stream}, ChaChaPoly1305, KeyInit, XChaCha20Poly1305};
use chacha20poly1305::aead::stream::{Decryptor, Encryptor, StreamBE32};
use chacha20poly1305::consts::U24;
use rand_core::{OsRng, RngCore};

pub fn generate_nonce() -> [u8; 24] {
    let mut nonce = [0u8; 24];
    OsRng.fill_bytes(&mut nonce);

    return nonce;
}

struct EncryptedStream {
    key: [u8; 32],
    nonce: [u8; 24],
    encryptor: Encryptor<ChaChaPoly1305<XChaCha20, U24>, StreamBE32<ChaChaPoly1305<XChaCha20, U24>>>,
    decryptor: Decryptor<ChaChaPoly1305<XChaCha20, U24>, StreamBE32<ChaChaPoly1305<XChaCha20, U24>>>,
    writer: Box<dyn Write>,
    reader: Box<dyn Read>
}

impl EncryptedStream {
    const BUFFER_LEN: usize = 500;

    pub fn new(key: [u8; 32], nonce: [u8; 24], writer: Box<dyn Write>, reader: Box<dyn Read>) -> Self {
        let encryptor = stream::EncryptorBE32::from_aead(XChaCha20Poly1305::new(&key.into()), nonce.as_ref().into());
        let decryptor = stream::DecryptorBE32::from_aead(XChaCha20Poly1305::new(&key.into()), nonce.as_ref().into());

        Self {
            key,
            nonce,
            encryptor,
            decryptor,
            writer,
            reader
        }
    }

    pub fn write_last(mut self, write_buffer: &[u8]) -> io::Result<usize> {
        let ciphertext = self.encryptor.encrypt_last(write_buffer);

        return match ciphertext {
            Ok(ciphertext) => self.writer.write(&ciphertext),
            Err(error) => Err(Error::new(ErrorKind::Other, format!("An error occurred while trying to encrypt stream buffer {}", error)))
        }
    }
}

impl Read for EncryptedStream {
    fn read(&mut self, read_buffer: &mut [u8]) -> io::Result<usize> {
        const BUFFER_LEN: usize = 500 + 16;
        let mut buffer = [0u8; BUFFER_LEN];
        let mut size: usize = 0;

        let read_bytes = self.reader.read(&mut buffer);

        if let Ok(read_bytes) = read_bytes {
            size += read_bytes;

            let decrypted_message_part = self.decryptor.decrypt_next(buffer.as_slice());

            if let Ok(decrypted_message_part) = decrypted_message_part {
                read_buffer.write(decrypted_message_part);
            } else if let Err(error) = decrypted_message_part {
                return Err(Error::new(ErrorKind::Other, format!("An error occurred while trying to decrypt stream buffer {}", error)));
            }
        }

        return Ok(size);
    }
}

impl Write for EncryptedStream {
    fn write(&mut self, write_buffer: &[u8]) -> io::Result<usize> {
        let ciphertext = self.encryptor.encrypt_next(write_buffer);

        return match ciphertext {
            Ok(ciphertext) => self.writer.write(&ciphertext),
            Err(error) => Err(Error::new(ErrorKind::Other, format!("An error occurred while trying to encrypt stream buffer {}", error)))
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}