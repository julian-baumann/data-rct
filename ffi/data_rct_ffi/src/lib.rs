uniffi::include_scaffolding!("data_rct");

use std::cell::{Cell, RefCell};
use std::io;
use std::io::Read;
use std::sync::Arc;
pub use data_rct::discovery::{DeviceInfo, Discovery, DiscoveryMethod, DiscoverySetupError, DiscoveryDelegate};
pub use data_rct::encryption::{EncryptedStream};
pub use data_rct::transmission::{Transmission};

#[derive(Debug, thiserror::Error)]
pub enum ExternalIOError {
    #[error("IO Error: {reason}")]
    IOError { reason: String }
}

impl From<io::Error> for ExternalIOError {
    fn from(error: io::Error) -> Self {
        return ExternalIOError::IOError { reason: error.to_string() }
    }
}

trait UniffiReadWrite {
    fn read_bytes(self: Arc<Self>, buffer: &Vec<u8>) -> Result<u64, ExternalIOError>;
    fn write_bytes(self: Arc<Self>, write_buffer: Vec<u8>) -> Result<u64, ExternalIOError>;
    fn flush_bytes(self: Arc<Self>) -> Result<(), ExternalIOError>;
}

impl UniffiReadWrite for EncryptedStream {
    fn read_bytes(self: Arc<Self>, buffer: &Vec<u8>) -> Result<u64, ExternalIOError> {
        let mut mutable_self = RefCell::new(self);
        let buffer = Arc::new(Cell::new(buffer));

        return Ok(mutable_self.get_mut().read(buffer.get_mut())? as u64);
    }

    fn write_bytes(self: Arc<Self>, write_buffer: Vec<u8>) -> Result<u64, ExternalIOError> {
        return Ok(io::Write::write(Arc::get_mut(&mut self).unwrap(), write_buffer.as_slice())? as u64);
    }

    fn flush_bytes(self: Arc<Self>) -> Result<(), ExternalIOError> {
        return Ok(io::Write::flush(Arc::get_mut(&mut self).unwrap())?);
    }
}
