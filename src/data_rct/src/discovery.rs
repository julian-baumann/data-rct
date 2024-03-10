use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use protocol::DiscoveryDelegate;
use protocol::discovery::{DeviceConnectionInfo, DeviceDiscoveryMessage, Device};
use protocol::discovery::device_discovery_message::Content;
use protocol::prost::Message;
use crate::errors::DiscoverySetupError;
use crate::init_logger;

pub trait BleDiscoveryImplementationDelegate: Send + Sync + Debug {
    fn start_scanning(&self);
    fn stop_scanning(&self);
}

static DISCOVERED_DEVICES: OnceLock<RwLock<HashMap<String, DeviceConnectionInfo>>> = OnceLock::new();

pub struct Discovery {
    pub ble_discovery_implementation: Option<Box<dyn BleDiscoveryImplementationDelegate>>,
    discovery_delegate: Option<Arc<Mutex<Box<dyn DiscoveryDelegate>>>>
}

impl Discovery {
    pub fn new(delegate: Option<Box<dyn DiscoveryDelegate>>) -> Result<Self, DiscoverySetupError> {
        init_logger();

        DISCOVERED_DEVICES.get_or_init(|| RwLock::new(HashMap::new()));

        let callback_arc = match delegate {
            Some(callback) => Some(Arc::new(Mutex::new(callback))),
            None => None
        };

        Ok(Self {
            ble_discovery_implementation: None,
            discovery_delegate: callback_arc
        })
    }

    pub fn get_connection_details(device: Device) -> Option<DeviceConnectionInfo> {
        Some(DISCOVERED_DEVICES.get()?.read().ok()?.get(&device.id)?.clone())
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

    pub fn parse_discovery_message(&mut self, data: Vec<u8>, ble_uuid: Option<String>) {
        let discovery_message = DeviceDiscoveryMessage::decode_length_delimited(data.as_slice());

        let Ok(discovery_message) = discovery_message else {
            return;
        };

        match discovery_message.content {
            None => {}
            Some(Content::DeviceConnectionInfo(device_connection_info)) => {
                let Some(device) = &device_connection_info.device else {
                    return;
                };

                let mut device_connection_info = device_connection_info.clone();

                if let Some(ble_uuid) = ble_uuid {
                    if let Some(mut ble_info) = device_connection_info.ble {
                        ble_info.uuid = ble_uuid;
                        device_connection_info.ble = Some(ble_info);
                    }
                }

                if !DISCOVERED_DEVICES.get().unwrap().read().unwrap().contains_key(&device.id) {
                    DISCOVERED_DEVICES.get().unwrap().write().unwrap().insert(device.id.clone(), device_connection_info.clone());
                    self.add_discovered_device(device.clone());
                }
            }
            Some(Content::OfflineDeviceId(device_id)) => {
                DISCOVERED_DEVICES.get().unwrap().write().unwrap().remove(&device_id);
                self.remove_discovered_device(device_id);
            }
        };
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
