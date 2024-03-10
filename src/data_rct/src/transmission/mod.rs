use thiserror::Error;

pub mod tcp;

#[derive(Error, Debug)]
pub enum TransmissionSetupError {
    #[error("Unable to start TCP server: {error}")]
    UnableToStartTcpServer { error: String },
}
