use std::sync::Arc;

pub use data_rct::{nearby::{BleServerImplementationDelegate, L2CapDelegate, NearbyConnectionDelegate, NearbyServer, SendProgressDelegate}, Device};
use data_rct::protocol::discovery::{BluetoothLeConnectionInfo, DeviceDiscoveryMessage, TcpConnectionInfo};
use data_rct::protocol::discovery::device_discovery_message::Content;
use data_rct::protocol::prost::Message;
pub use data_rct::stream::NativeStreamDelegate;
pub use data_rct::errors::*;
use tokio::sync::RwLock;

struct InternalNearbyServerVariables {
    discovery_message: Option<Vec<u8>>
}

#[derive(uniffi::Object)]
pub struct InternalNearbyServer {
    handler: NearbyServer,
    mut_variables: Arc<RwLock<InternalNearbyServerVariables>>
}

#[uniffi::export(async_runtime = "tokio")]
impl InternalNearbyServer {
    #[uniffi::constructor]
    pub fn new(my_device: Device, file_storage: String, delegate: Box<dyn NearbyConnectionDelegate>) -> Self {
        let server = NearbyServer::new(my_device, file_storage, delegate);

        Self {
            handler: server,
            mut_variables: Arc::new(RwLock::new(InternalNearbyServerVariables {
                discovery_message: None
            }))
        }
    }

    pub fn add_l2_cap_client(&self, delegate: Box<dyn L2CapDelegate>) {
        self.handler.add_l2_cap_client(delegate);
    }

    pub fn add_ble_implementation(&self, ble_implementation: Box<dyn BleServerImplementationDelegate>) {
        self.handler.add_bluetooth_implementation(ble_implementation);
    }

    pub fn change_device(&self, new_device: Device) {
        self.handler.change_device(new_device);
    }

    pub fn set_ble_connection_details(&self, ble_details: BluetoothLeConnectionInfo) {
        self.handler.set_bluetooth_le_details(ble_details)
    }

    pub fn set_tcp_details(&self, tcp_details: TcpConnectionInfo) {
        self.handler.set_tcp_details(tcp_details)
    }

    pub async fn get_advertisement_data(&self) -> Vec<u8> {
        if self.mut_variables.read().await.discovery_message.is_none() {
            if self.handler.variables.read().await.advertise {
                let message = Some(DeviceDiscoveryMessage {
                    content: Some(
                        Content::DeviceConnectionInfo(
                            self.handler.variables
                                .read()
                                .await
                                .device_connection_info.clone()
                        )
                    ),
                }.encode_length_delimited_to_vec());

                self.mut_variables.write().await.discovery_message = message;
            }
        }

        if let Some(discovery_message) = &self.mut_variables.read().await.discovery_message {
            return discovery_message.clone();
        }

        return vec![];
    }

    pub async fn start(&self) {
        self.handler.start().await;
    }

    pub fn handle_incoming_ble_connection(&self, connection_id: String, native_stream: Box<dyn NativeStreamDelegate>) {
        return self.handler.handle_incoming_ble_connection(connection_id, native_stream);
    }

    pub async fn send_file(&self, receiver: Device, file_path: String, progress_delegate: Option<Box<dyn SendProgressDelegate>>) -> Result<(), ConnectErrors> {
        return self.handler.send_file(receiver, file_path, progress_delegate).await;
    }

    pub fn stop(&self) {
        self.handler.stop();
    }

    pub fn handle_incoming_connection(&self, native_stream_handle: Box<dyn NativeStreamDelegate>) {
        self.handler.handle_incoming_connection(native_stream_handle);
    }
}
