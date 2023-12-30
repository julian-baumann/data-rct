use std::sync::{Arc, Mutex, OnceLock};
use protocol::discovery::Device;
use protocol::DiscoveryDelegate;
use crate::platforms::apple::peripheral_manager::PeripheralManager;

mod peripheral_manager;
mod ffi;
mod constants;
mod converter;
mod events;
mod central_manager;

pub(crate) fn get_discovered_devices_mutex() -> &'static Mutex<Vec<Device>> {
    static ARRAY: OnceLock<Mutex<Vec<Device>>> = OnceLock::new();
    return ARRAY.get_or_init(|| Mutex::new(Vec::new()));
}

pub(crate) fn add_new_device(device: Device) -> bool {
    let devices = get_discovered_devices_mutex()
        .get_mut()
        .expect("Failed to unwrap get_mut() on discovered devices");

    for existing_device in devices {
        if existing_device.id == device.id {
            return false;
        }
    }

    devices.push(device);

    return true;
}

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

    pub fn get_devices(&self) -> Vec<&Device> {
        return get_discovered_devices_mutex()
            .lock()
            .expect("Failed to lock discovered_devices")
            .iter()
            .collect();
    }
}
