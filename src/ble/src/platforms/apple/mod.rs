use std::sync::{Arc, Mutex};
use protocol::discovery::Device;
use protocol::DiscoveryDelegate;
use crate::platforms::apple::peripheral_manager::PeripheralManager;

mod peripheral_manager;
mod ffi;
mod constants;
mod converter;
mod events;
mod central_manager;

static mut DISCOVERED_DEVICES: Mutex<Vec<Device>> = Mutex::new(Vec::new());
static mut DISCOVERY_DELEGATE: Option<Arc<Mutex<Box<dyn DiscoveryDelegate>>>> = None;

pub struct BleDiscovery {
    peripheral_manager: PeripheralManager
}

impl BleDiscovery {
    pub fn new(discovery_delegate: Option<Arc<Mutex<Box<dyn DiscoveryDelegate>>>>) -> Self {
        unsafe {
            if let Some(discovery_delegate) = discovery_delegate {
                DISCOVERY_DELEGATE.replace(discovery_delegate);
            }
        }

        BleDiscovery {
            peripheral_manager: PeripheralManager::new()
        }
    }

    pub fn is_powered_on(&self) -> bool {
        self.peripheral_manager.is_powered()
    }

    pub fn is_discovering(&self) -> bool {
        self.peripheral_manager.is_advertising()
    }

    pub fn start_discovering_devices(&self) {
        self.peripheral_manager.configure_service();
        self.peripheral_manager.start_advertising();
    }

    pub fn stop_discovering_devices(&self) {
        self.peripheral_manager.stop_advertising();
    }

    pub fn get_devices(&self) -> Vec<Device> {
        unsafe {
            return DISCOVERED_DEVICES.lock()
                .expect("Failed to lock DISCOVERED_DEVICES")
                .to_vec()
        }
    }
}
