use std::io;
use std::sync::{Arc, RwLock};

pub use data_rct::{BLE_CHARACTERISTIC_UUID, BLE_SERVICE_UUID, ClipboardTransferIntent};
pub use data_rct::connection_request::ConnectionRequest;
pub use data_rct::Device;
pub use data_rct::discovery::{BleDiscoveryImplementationDelegate, Discovery, DiscoverySetupError};
pub use data_rct::DiscoveryDelegate as DeviceListUpdateDelegate;
pub use data_rct::encryption::EncryptedStream;
pub use data_rct::nearby::{SendProgressState, ProgressDelegate, BleServerImplementationDelegate, ConnectErrors, L2CAPDelegate, NearbyConnectionDelegate, NearbyServer};
pub use data_rct::nearby::ConnectionIntentType;
pub use data_rct::protocol::communication::FileTransferIntent;
use data_rct::protocol::discovery::{BluetoothLeConnectionInfo, DeviceDiscoveryMessage, TcpConnectionInfo};
use data_rct::protocol::discovery::device_discovery_message::Content;
use data_rct::protocol::prost::Message;
pub use data_rct::stream::IncomingErrors;
pub use data_rct::transmission::TransmissionSetupError;

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

pub fn get_ble_service_uuid() -> String {
    return BLE_SERVICE_UUID.to_string();
}

pub fn get_ble_characteristic_uuid() -> String {
    return BLE_CHARACTERISTIC_UUID.to_string();
}

#[derive(uniffi::Object)]
pub struct InternalNearbyServer {
    handler: Arc<tokio::sync::RwLock<NearbyServer>>
}

#[uniffi::export(async_runtime = "tokio")]
impl InternalNearbyServer {

    #[uniffi::constructor]
    pub fn new(my_device: Device, file_storage: String, delegate: Box<dyn NearbyConnectionDelegate>) -> Self {
        let server = NearbyServer::new(my_device, file_storage, delegate);
        let server = Arc::new(tokio::sync::RwLock::new(server));

        Self {
            handler: server
        }
    }

    pub fn add_l2cap_client(&self, delegate: Box<dyn L2CAPDelegate>) {
        self.handler.blocking_write().add_l2cap_client(delegate);
    }

    pub fn add_ble_implementation(&self, ble_implementation: Box<dyn BleServerImplementationDelegate>) {
        self.handler.blocking_write().add_bluetooth_implementation(ble_implementation);
    }

    pub fn change_device(&self, new_device: Device) {
        self.handler.blocking_write().change_device(new_device);
    }

    pub fn set_ble_connection_details(&self, ble_details: BluetoothLeConnectionInfo) {
        self.handler.blocking_write().set_bluetooth_le_details(ble_details)
    }

    pub fn set_tcp_details(&self, tcp_details: TcpConnectionInfo) {
        self.handler.blocking_write().set_tcp_details(tcp_details)
    }

    pub async fn get_advertisement_data(&self) -> Vec<u8> {
        if self.handler.read().await.advertise {
            return DeviceDiscoveryMessage {
                content: Some(
                    Content::DeviceConnectionInfo(
                        self.handler
                            .read()
                            .await
                            .device_connection_info.clone()
                    )
                ),
            }.encode_length_delimited_to_vec();
        }

        return DeviceDiscoveryMessage {
            content: Some(
                Content::OfflineDeviceId(
                    self.handler
                        .read()
                        .await
                        .device_connection_info.device
                        .as_ref()
                        .expect("Device not set!")
                        .id.clone()
                )
            ),
        }.encode_length_delimited_to_vec();
    }

    pub async fn start(&self) {
        self.handler.write().await.start().await.expect("Failed to start server");
    }

    pub async fn send_file(&self, receiver: Device, file_path: String, progress_delegate: Option<Box<dyn ProgressDelegate>>) -> Result<(), ConnectErrors> {
        return self.handler.read().await.send_file(receiver, file_path, progress_delegate).await;
    }

    pub async fn stop(&self) {
        println!("Stopping...");
        self.handler.write().await.stop();
        println!("Stopped");
    }
}

pub struct InternalDiscovery {
    handler: Arc<RwLock<Discovery>>
}

impl InternalDiscovery {
    pub fn new(delegate: Option<Box<dyn DeviceListUpdateDelegate>>) -> Result<Self, DiscoverySetupError> {

        Ok(Self {
            handler: Arc::new(RwLock::new(Discovery::new(delegate)?))
        })
    }

    pub fn add_ble_implementation(&self, implementation: Box<dyn BleDiscoveryImplementationDelegate>) {
        self.handler.write().expect("Failed to lock handler").add_ble_implementation(implementation);
    }

    pub fn start(&self) {
        if let Some(ble_discovery_implementation) = &self.handler.read().expect("Failed to lock handler").ble_discovery_implementation {
            ble_discovery_implementation.start_scanning();
        }
    }

    pub fn stop(&self) {
        if let Some(ble_discovery_implementation) = &self.handler.read().expect("Failed to lock handler").ble_discovery_implementation {
            ble_discovery_implementation.stop_scanning();
        }
    }

    pub fn parse_discovery_message(&self, data: Vec<u8>, ble_uuid: Option<String>) {
        self.handler.write().expect("Failed to lock handler").parse_discovery_message(data, ble_uuid);
    }
}
//
// trait UniffiReadWrite {
//     fn write_bytes(&self, write_buffer: Vec<u8>) -> Result<u64, ExternalIOError>;
//     fn flush_bytes(&self) -> Result<(), ExternalIOError>;
// }
//
// impl UniffiReadWrite for EncryptedStream {
//     fn write_bytes(&self, buffer: Vec<u8>) -> Result<u64, ExternalIOError> {
//         return Ok(
//             self.write_immutable(buffer.as_slice())? as u64
//         );
//     }
//
//     fn flush_bytes(&self) -> Result<(), ExternalIOError> {
//         return Ok(self.flush_immutable()?);
//     }
// }

uniffi::include_scaffolding!("data_rct");
