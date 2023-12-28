use std::cell::OnceCell;
use std::collections::HashMap;
use std::sync::{Once, OnceLock};
use lazy_static::lazy_static;
use protocol::discovery::Device;
use crate::CorePeripheral;
use crate::platforms::apple::peripheral_manager::PeripheralManager;

mod peripheral_manager;
mod ffi;
mod constants;
mod converter;
mod events;

static DISCOVERED_DEVICES: OnceLock<Vec<Device>> = OnceLock::new();

pub struct Discovery {
    peripheral_manager: PeripheralManager
}

impl Discovery {
    pub fn new() -> Self {
        Discovery {
            peripheral_manager: PeripheralManager::new(),
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
        return DISCOVERED_DEVICES.get()
            .expect("Failed to lock DISCOVERED_DEVICES")
            .to_vec()
    }
}
