use std::sync::{Arc, Mutex, OnceLock};
use protocol::discovery::{Device, DeviceDiscoveryMessage};
use protocol::discovery::device_discovery_message::DeviceData;
use protocol::DiscoveryDelegate;
use protocol::prost::Message;
use crate::apple::peripheral_manager::PeripheralManager;

mod peripheral_manager;
mod ffi;
mod constants;
mod converter;
mod events;
mod central_manager;

static mut DISCOVERY_MESSAGE: Vec<u8> = vec![];

pub struct BleAdvertisement {
    peripheral_manager: PeripheralManager,
    my_device: Device
}

impl BleAdvertisement {
    pub fn new(device: Device) -> Self {

        BleAdvertisement {
            peripheral_manager: PeripheralManager::new(),
            my_device: device
        }
    }

    pub fn is_powered_on(&self) -> bool {
        self.peripheral_manager.is_powered()
    }

    pub fn is_advertising(&self) -> bool {
        self.peripheral_manager.is_advertising()
    }

    pub fn start_advertising(&self) {
        unsafe {
            let message = DeviceDiscoveryMessage {
                device_data: Some(DeviceData::Device(self.my_device.clone()))
            };

            DISCOVERY_MESSAGE = message.encode_length_delimited_to_vec();
        }

        self.peripheral_manager.configure_service();
        self.peripheral_manager.start_advertising();
    }

    pub fn stop_advertising(&self) {
        unsafe {
            let message = DeviceDiscoveryMessage {
                device_data: Some(DeviceData::DeviceId(self.my_device.id.clone()))
            };

            DISCOVERY_MESSAGE = message.encode_length_delimited_to_vec();
        }

        self.peripheral_manager.stop_advertising();
    }
}
