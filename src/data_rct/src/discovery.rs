use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use protocol::{DiscoveryDelegate};
use protocol::discovery::{Device, DeviceDiscoveryMessage};
use protocol::discovery::device_discovery_message::DeviceData;
use protocol::prost::Message;

#[derive(Error, Debug)]
pub enum DiscoverySetupError {
    #[error("Unable to setup UDP Discovery")]
    UnableToSetupUdp,

    #[error("Unable to setup MDNS-SD Discovery")]
    UnableToSetupMdns
}

pub trait BleDiscoveryImplementationDelegate: Send + Sync + Debug {
    fn start_scanning(&self);
    fn stop_scanning(&self);
}

pub struct Discovery {
    pub ble_discovery_implementation: Option<Box<dyn BleDiscoveryImplementationDelegate>>,
    discovery_delegate: Option<Arc<Mutex<Box<dyn DiscoveryDelegate>>>>,
    discovered_devices: HashMap<String, Device>
}

impl Discovery {
    pub fn new(delegate: Option<Box<dyn DiscoveryDelegate>>) -> Result<Self, DiscoverySetupError> {
        let callback_arc = match delegate {
            Some(callback) => Some(Arc::new(Mutex::new(callback))),
            None => None
        };

        Ok(Self {
            ble_discovery_implementation: None,
            discovery_delegate: callback_arc,
            discovered_devices: HashMap::new()
        })
    }

    pub fn add_ble_implementation(&mut self, implementation: Box<dyn BleDiscoveryImplementationDelegate>) {
        self.ble_discovery_implementation = Some(implementation)
    }

    pub fn start(&self) {
        if let Some(ble_discovery_implementation) = &self.ble_discovery_implementation {
            ble_discovery_implementation.start_scanning();
        }
    }

    pub fn stop(&self) {
        if let Some(ble_discovery_implementation) = &self.ble_discovery_implementation {
            ble_discovery_implementation.stop_scanning();
        }
    }

    pub fn parse_discovery_message(&mut self, data: Vec<u8>) {
        let discovery_message = DeviceDiscoveryMessage::decode_length_delimited(data.as_slice());

        let Ok(discovery_message) = discovery_message else {
            return;
        };

        match discovery_message.device_data {
            None => {}
            Some(DeviceData::Device(device)) => {
                if !self.discovered_devices.contains_key(&device.id) {
                    self.discovered_devices.insert(device.id.clone(), device.clone());
                    self.add_discovered_device(device);
                }
            }
            Some(DeviceData::DeviceId(device_id)) => {
                self.discovered_devices.remove(&device_id);
                self.remove_discovered_device(device_id);
            }
        }
    }

    fn add_discovered_device(&self, device: Device) {
        if let Some(discovery_delegate) = &self.discovery_delegate {
            discovery_delegate.lock().expect("Failed to lock discovery_delegate").device_added(device);
        }
    }

    fn remove_discovered_device(&self, device_id: String) {
        if let Some(discovery_delegate) = &self.discovery_delegate {
            discovery_delegate.lock().expect("Failed to lock discovery_delegate").device_removed(device_id);
        }
    }
}
