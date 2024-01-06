uniffi::include_scaffolding!("data_rct");

use std::io;
use std::sync::{Arc, Mutex};
use data_rct::{BLE_CHARACTERISTIC_UUID, BLE_SERVICE_UUID};
pub use data_rct::Device;
pub use data_rct::encryption::{EncryptedStream};
pub use data_rct::stream::{ConnectErrors, IncomingErrors};
pub use data_rct::DiscoveryDelegate as DeviceListUpdateDelegate;
pub use data_rct::discovery::{Discovery, DiscoverySetupError, BleDiscoveryImplementationDelegate};
pub use data_rct::nearby::{BleServerImplementationDelegate, NearbyServer};
use data_rct::protocol::discovery::device_discovery_message::DeviceData;
use data_rct::protocol::discovery::DeviceDiscoveryMessage;
use data_rct::protocol::prost::Message;
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

struct InternalNearbyServer {
    handler: Arc<Mutex<NearbyServer>>
}

impl InternalNearbyServer {
    pub fn new(my_device: Device) -> Result<Self, TransmissionSetupError> {
        let server = NearbyServer::new(my_device)?;
        let server = Arc::new(Mutex::new(server));

        Ok(Self {
            handler: server
        })
    }

    pub fn add_ble_implementation(&self, ble_implementation: Box<dyn BleServerImplementationDelegate>) {
        println!("Adding implementation...");
        self.handler.lock().expect("Failed to lock NearbyServer handler").add_bluetooth_implementation(ble_implementation);
        println!("Added implementation");
    }
    
    pub fn change_device(&self, new_device: Device) {
        self.handler.lock().expect("Failed to lock NearbyServer handler").my_device = new_device;
    }

    pub fn get_advertisement_data(&self) -> Vec<u8> {
        println!("Test 1");

        let handler = self.handler.lock().expect("Failed to lock NearbyServer handler");

        println!("Test 2");

        if handler.advertise {
            return DeviceDiscoveryMessage {
                device_data: Some(DeviceData::Device(handler.my_device.clone())),
            }.encode_length_delimited_to_vec();
        }

        return DeviceDiscoveryMessage {
            device_data: Some(DeviceData::DeviceId(handler.my_device.id.clone())),
        }.encode_length_delimited_to_vec();
    }

    pub fn start(&self) {
        println!("Starting...");
        self.handler.lock().expect("Failed to lock NearbyServer handler").start();
        println!("Started");
    }

    pub fn stop(&self) {
        println!("Stopping...");
        self.handler.lock().expect("Failed to lock NearbyServer handler").stop();
        println!("Stopped");
    }
}


pub struct InternalDiscovery {
    handler: Arc<Mutex<Discovery>>
}

impl InternalDiscovery {
    pub fn new(delegate: Option<Box<dyn DeviceListUpdateDelegate>>) -> Result<Self, DiscoverySetupError> {

        Ok(Self {
            handler: Arc::new(Mutex::new(Discovery::new(delegate)?))
        })
    }

    pub fn add_ble_implementation(&self, implementation: Box<dyn BleDiscoveryImplementationDelegate>) {
        self.handler.lock().expect("Failed to lock handler").add_ble_implementation(implementation);
    }

    pub fn start(&self) {
        if let Some(ble_discovery_implementation) = &self.handler.lock().expect("Failed to lock handler").ble_discovery_implementation {
            ble_discovery_implementation.start_scanning();
        }
    }

    pub fn stop(&self) {
        if let Some(ble_discovery_implementation) = &self.handler.lock().expect("Failed to lock handler").ble_discovery_implementation {
            ble_discovery_implementation.stop_scanning();
        }
    }

    pub fn parse_discovery_message(&self, data: Vec<u8>) {
        self.handler.lock().expect("Failed to lock handler").parse_discovery_message(data);
    }
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
// trait TransmissionFfi {
//     fn connect_to_device(&self, recipient: Device) -> Result<Arc<EncryptedStream>, ConnectErrors>;
// }
//
// impl TransmissionFfi for Transmission {
//     fn connect_to_device(&self, recipient: Device) -> Result<Arc<EncryptedStream>, ConnectErrors> {
//         return match self.open(&recipient) {
//             Ok(result) => Ok(Arc::new(result)),
//             Err(error) => Err(error)
//         };
//     }
// }
