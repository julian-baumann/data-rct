use std::io;
use std::io::{Read, Write};
use std::string::FromUtf8Error;
use downcast_rs::{DowncastSync, impl_downcast};
use thiserror::Error;
use anyhow::{Result};

#[derive(Error, Debug)]
pub enum IncomingErrors {
    #[error("Unknown reading error: {0}")]
    UnknownReadError(io::Error),

    #[error("Error while trying to convert utf8-sequence to string: {0}")]
    StringConversionError(FromUtf8Error),

    #[error("Missing protocol version")]
    MissingProtocolVersion,

    #[error("Invalid version")]
    InvalidVersion,

    #[error("Invalid uuid")]
    InvalidUUID,

    #[error("Invalid foreign public key")]
    InvalidForeignPublicKey,

    #[error("Error sending public key")]
    ErrorSendingPublicKey,

    #[error("Invalid nonce")]
    InvalidNonce,

    #[error("Encryption error")]
    EncryptionError,

    #[error("Invalid sender-id")]
    InvalidSenderId,

    #[error("Invalid sender-name")]
    InvalidSenderName,

    #[error("Recipient rejected the transmission")]
    Rejected,
}


#[derive(Error, Debug)]
pub enum ConnectErrors {
    #[error("Unknown write error: {0}")]
    UnknownWriteError(io::Error),

    #[error("Unknown read error: {0}")]
    UnknownReadError(io::Error),

    #[error("Invalid socket address")]
    InvalidSocketAddress,

    #[error("Unable to open socket")]
    CouldNotOpenSocket,

    #[error("Error while trying to encrypt stream: {0}")]
    EncryptionError(String),

    #[error("Invalid foreign public key. {0}")]
    InvalidForeignPublicKey(String),

    #[error("Recipient rejected the transmission")]
    Rejected,
}

pub fn check_result(result: core::result::Result<usize, io::Error>, error: IncomingErrors, expected_length: Option<usize>) -> Option<IncomingErrors> {
    if let Err(error) = result {
        return Some(IncomingErrors::UnknownReadError(error));
    }

    if let Some(expected_length) = expected_length {
        if let Ok(bytes_read) = result {
            if bytes_read > expected_length {
                return Some(error);
            }
        }
    }

    return None;
}

pub trait StreamRead: Read {
    fn read_u8(&mut self) -> std::result::Result<u8, io::Error> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_and_get_value<'a>(&mut self, length: usize, error: IncomingErrors) -> Result<Vec<u8>, IncomingErrors> {
        let mut buffer = vec![0u8; length];
        let slice_buffer = buffer.as_mut_slice();
        let result = self.read(slice_buffer);

        if let Some(error) = check_result(result, error, Some(length)) {
            return Err(error);
        }

        return Ok(slice_buffer.to_owned());
    }

    fn read_string(&mut self, length: usize, error: IncomingErrors) -> Result<String, IncomingErrors> {
        let value = match self.read_and_get_value(length, error) {
            Ok(value) => value,
            Err(error) => return Err(error)
        };

        let value_as_string = match String::from_utf8(value) {
            Ok(value) => value,
            Err(error) => return Err(IncomingErrors::StringConversionError(error))
        };

        return Ok(value_as_string);
    }
}

pub trait StreamWrite: Write {
    fn write_stream(&mut self, buffer: &[u8]) -> Result<usize, ConnectErrors> {
        return match self.write(buffer) {
            Ok(value) => Ok(value),
            Err(error) => Err(ConnectErrors::UnknownWriteError(error))
        }
    }
}

pub trait Stream: StreamRead + StreamWrite + DowncastSync {
}
impl_downcast!(sync Stream);
