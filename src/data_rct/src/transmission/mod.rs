use std::error::Error;
use thiserror::Error;
use crate::stream::Stream;

pub mod tcp;


#[derive(Error, Debug)]
pub enum TransmissionSetupError {
    #[error("Unable to start TCP server: {error}")]
    UnableToStartTcpServer { error: String }
}

pub trait DataTransmission {
    fn new() -> Result<Self, Box<dyn Error>> where Self: Sized;
    fn accept(&self) -> Option<Box<dyn Stream>>;
}
