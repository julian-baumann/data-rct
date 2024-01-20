use std::{io};
use std::io::{Error, Read, Write};
use std::io::ErrorKind::Other;
use std::iter::repeat;
use rand_core::{OsRng};
use chacha20::{XChaCha20};
use chacha20::cipher::{KeyIvInit, StreamCipher};

pub fn generate_key() -> [u8; 32] {
    let key = XChaCha20::generate_key(&mut OsRng);

    return key.into();
}

pub fn generate_iv() -> [u8; 24] {
    let nonce = XChaCha20::generate_iv(&mut OsRng);

    return nonce.into();
}

pub struct EncryptedStream<TStream> where TStream : Read + Write {
    pub cipher: XChaCha20,
    pub raw_stream: TStream
}

impl<TStream> EncryptedStream<TStream> where TStream : Read + Write {
    pub fn new(key: [u8; 32], iv: [u8; 24], stream: TStream) -> Self {
        let cipher = XChaCha20::new(&key.into(), &iv.into());

        Self {
            cipher,
            raw_stream: stream
        }
    }

    // fn read_available(&mut self) -> io::Result<usize>  {
    //     let mut total_bytes_read: usize = 0;
    //
    //     loop {
    //         let mut buffer = [0; BUFFER_SIZE];
    //         let read_bytes = self.raw_stream.read(&mut buffer);
    //
    //         let Ok(read_bytes) = read_bytes else {
    //             return Err(read_bytes.unwrap_err());
    //         };
    //
    //         total_bytes_read += read_bytes;
    //
    //         self.encrypted_read_buffer.put(buffer.as_slice());
    //
    //         if read_bytes < BUFFER_SIZE {
    //             break;
    //         }
    //     }
    //
    //     return Ok(total_bytes_read);
    // }
}

impl<TStream> Read for EncryptedStream<TStream> where TStream : Read + Write {
    fn read(&mut self, read_buffer: &mut [u8]) -> io::Result<usize> {
        let mut buffer: Vec<u8> = repeat(0).take(read_buffer.len()).collect();
        let read_bytes = self.raw_stream.read(&mut buffer).expect("Failed to read from encrypted buffer");

        if read_bytes <= 0 {
            return Ok(0);
        }

        match self.cipher.apply_keystream_b2b(&buffer[..read_bytes], &mut read_buffer[..read_bytes]) {
            Ok(_) => {}
            Err(error) => return Err(Error::new(Other, error.to_string()))
        };

        return Ok(read_bytes);
    }
}

impl<TStream> Write for EncryptedStream<TStream> where TStream : Read + Write {
    fn write(&mut self, write_buffer: &[u8]) -> io::Result<usize> {
        let mut buffer: Vec<u8> = repeat(0).take(write_buffer.len()).collect();
        let ciphertext = self.cipher.apply_keystream_b2b(write_buffer, &mut buffer);

        if let Ok(()) = ciphertext {
            return self.raw_stream.write(&buffer);
        }

        return Ok(0);
    }

    fn flush(&mut self) -> io::Result<()> {
        return self.raw_stream.flush();
    }
}
