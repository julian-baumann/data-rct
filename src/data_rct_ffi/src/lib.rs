uniffi::include_scaffolding!("data_rct");

use std::io;
use std::io::Read;
use std::sync::Arc;
pub use data_rct::discovery::{DeviceInfo, Discovery, DiscoveryMethod, DiscoverySetupError, DiscoveryDelegate};
pub use data_rct::encryption::{EncryptedStream};
pub use data_rct::transmission::{Transmission, TransmissionSetupError, TransmissionRequest};
pub use data_rct::stream::{ConnectErrors, IncomingErrors};


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

#[no_mangle]
pub unsafe extern fn read_unsafe(encrypted_stream: *mut EncryptedStream, buffer: *mut [u8]) -> u64 {
    let result = (*encrypted_stream).read(&mut (*buffer));

    return 0;
}

trait UniffiReadWrite {
    fn write_bytes(&self, write_buffer: Vec<u8>) -> Result<u64, ExternalIOError>;
    fn flush_bytes(&self) -> Result<(), ExternalIOError>;
}

impl UniffiReadWrite for EncryptedStream {
    fn write_bytes(&self, buffer: Vec<u8>) -> Result<u64, ExternalIOError> {
        return Ok(
            self.write_immutable(buffer.as_slice())? as u64
        );
    }

    fn flush_bytes(&self) -> Result<(), ExternalIOError> {
        return Ok(self.flush_immutable()?);
    }
}

trait TransmissionFfi {
    fn connect_to_device(&self, recipient: DeviceInfo) -> Result<Arc<EncryptedStream>, ConnectErrors>;
}

impl TransmissionFfi for Transmission {
    fn connect_to_device(&self, recipient: DeviceInfo) -> Result<Arc<EncryptedStream>, ConnectErrors> {
        return match self.open(&recipient) {
            Ok(result) => Ok(Arc::new(result)),
            Err(error) => Err(error)
        };
    }
}
