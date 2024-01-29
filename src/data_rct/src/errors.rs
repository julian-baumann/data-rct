use std::io;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConnectErrors {
    #[error("Peripheral is unreachable")]
    Unreachable,

    #[error("Failed to get connection details")]
    FailedToGetConnectionDetails,

    #[error("Peripheral declined the connection")]
    Declined,

    #[error("Failed to get TCP connection details")]
    FailedToGetTcpDetails,

    #[error("Failed to get socket address")]
    FailedToGetSocketAddress,

    #[error("Failed to open TCP stream")]
    FailedToOpenTcpStream,

    #[error("Failed to get BLE connection details")]
    FailedToGetBleDetails,

    #[error("No available internal BLE handler found.")]
    InternalBleHandlerNotAvailable,

    #[error("Failed to establish a BLE connection to the peripheral.")]
    FailedToEstablishBleConnection,

    #[error("Failed to encrypt stream: {error}")]
    FailedToEncryptStream { error: String },

    #[error("Failed to determine file size: {error}")]
    FailedToDetermineFileSize { error: String },

    #[error("Failed to get transfer request response: {error}")]
    FailedToGetTransferRequestResponse { error: String },
}

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
pub enum DiscoverySetupError {
    #[error("Unable to setup UDP Discovery")]
    UnableToSetupUdp,

    #[error("Unable to setup MDNS-SD Discovery")]
    UnableToSetupMdns
}