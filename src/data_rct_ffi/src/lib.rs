use std::io;
use std::sync::Arc;

pub use data_rct::connection_request::{
    ConnectionRequest, ReceiveProgressDelegate, ReceiveProgressState,
};
pub use data_rct::discovery::{BleDiscoveryImplementationDelegate, Discovery};
pub use data_rct::encryption::EncryptedStream;
pub use data_rct::errors::*;
pub use data_rct::nearby::ConnectionIntentType;
pub use data_rct::nearby::{
    BleServerImplementationDelegate, L2CapDelegate, NearbyConnectionDelegate, NearbyServer,
    SendProgressDelegate, SendProgressState,
};
pub use data_rct::protocol::communication::FileTransferIntent;
use data_rct::protocol::discovery::{BluetoothLeConnectionInfo, TcpConnectionInfo};
pub use data_rct::stream::NativeStreamDelegate;
pub use data_rct::transmission::TransmissionSetupError;
pub use data_rct::Device;
pub use data_rct::DiscoveryDelegate as DeviceListUpdateDelegate;
pub use data_rct::*;
pub use data_rct::{ClipboardTransferIntent, BLE_CHARACTERISTIC_UUID, BLE_SERVICE_UUID};

#[cfg(not(feature = "sync"))]
pub mod async_code;
#[cfg(feature = "sync")]
pub mod sync_code;

#[derive(Debug, thiserror::Error)]
pub enum ExternalIOError {
    #[error("IO Error: {reason}")]
    IOError { reason: String },
}

impl From<io::Error> for ExternalIOError {
    fn from(error: io::Error) -> Self {
        return ExternalIOError::IOError {
            reason: error.to_string(),
        };
    }
}

pub fn get_ble_service_uuid() -> String {
    return BLE_SERVICE_UUID.to_string();
}

pub fn get_ble_characteristic_uuid() -> String {
    return BLE_CHARACTERISTIC_UUID.to_string();
}

pub struct InternalDiscovery {
    handler: Arc<std::sync::RwLock<Discovery>>,
}

impl InternalDiscovery {
    pub fn new(
        delegate: Option<Box<dyn DeviceListUpdateDelegate>>,
    ) -> Result<Self, DiscoverySetupError> {
        Ok(Self {
            handler: Arc::new(std::sync::RwLock::new(Discovery::new(delegate)?)),
        })
    }

    pub fn add_ble_implementation(
        &self,
        implementation: Box<dyn BleDiscoveryImplementationDelegate>,
    ) {
        self.handler
            .write()
            .expect("Failed to lock handler")
            .add_ble_implementation(implementation);
    }

    pub fn start(&self) {
        if let Some(ble_discovery_implementation) = &self
            .handler
            .read()
            .expect("Failed to lock handler")
            .ble_discovery_implementation
        {
            ble_discovery_implementation.start_scanning();
        }
    }

    pub fn stop(&self) {
        if let Some(ble_discovery_implementation) = &self
            .handler
            .read()
            .expect("Failed to lock handler")
            .ble_discovery_implementation
        {
            ble_discovery_implementation.stop_scanning();
        }
    }

    pub fn parse_discovery_message(&self, data: Vec<u8>, ble_uuid: Option<String>) {
        self.handler
            .write()
            .expect("Failed to lock handler")
            .parse_discovery_message(data, ble_uuid);
    }
}

uniffi::include_scaffolding!("data_rct");
