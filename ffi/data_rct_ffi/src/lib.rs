uniffi::include_scaffolding!("data_rct");

pub use data_rct::discovery::{DeviceInfo, Discovery, DiscoveryMethod, DiscoverySetupError, DiscoveryDelegate};
pub use data_rct::encryption::{EncryptedStream};
pub use data_rct::transmission::{Transmission};

// #[derive(Debug, thiserror::Error)]
// enum IoError {
//     #[error("Read error {error}")]
//     ReadError { error: String },
//
//     #[error("Write error {error}")]
//     WriteError { error: String },
//
//     #[error("Flush error {error}")]
//     FlushError { error: String }
// }
//
// trait UniffiReadWrite {
//     fn read(&mut self, read_buffer: &mut [u8]) -> Result<usize, IoError>;
//     fn write(&mut self, write_buffer: &[u8]) -> Result<usize, IoError>;
//     fn flush(&mut self) -> Result<(), IoError>;
// }
//
// impl UniffiReadWrite for EncryptedStream {
//     fn read(&mut self, read_buffer: &mut [u8]) -> Result<usize, IoError> {
//         return match std::io::Read::read(&mut self, read_buffer) {
//             Ok(result) => Ok(result),
//             Err(error) => Err(IoError::ReadError { error: error.to_string() })
//         };
//     }
//
//     fn write(&mut self, write_buffer: &[u8]) -> Result<usize, IoError> {
//         return match std::io::Write::write(&mut self, write_buffer) {
//             Ok(result) => Ok(result),
//             Err(error) => Err(IoError::WriteError { error: error.to_string() })
//         };
//     }
//
//     fn flush(&mut self) -> Result<(), IoError> {
//         return match std::io::Write::flush(&mut self) {
//             Ok(result) => Ok(result),
//             Err(error) => Err(IoError::FlushError { error: error.to_string() })
//         };
//     }
// }
