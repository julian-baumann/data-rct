use std::collections::HashMap;
use std::thread;
use std::thread::JoinHandle;
use crossbeam_channel::Receiver;
use crate::udp_discovery::UdpDiscovery;

#[derive(Clone, PartialEq)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub port: u8,
    pub device_type: String,
    pub ip_address: String
}

pub struct Discovery {
    discovery_thread: Option<JoinHandle<()>>,
    pub my_device: DeviceInfo,
    discovered_devices: HashMap<String, DeviceInfo>,
    receiver: Receiver<HashMap<String, DeviceInfo>>
}

impl Discovery {
    pub fn new(my_device: DeviceInfo) -> Discovery {
        let device = my_device.clone();
        let (sender, receiver) = crossbeam_channel::unbounded();

        let mut discovery = UdpDiscovery::new(device, sender).unwrap();

        thread::spawn(move || {
            discovery.start_loop().ok();
        });

        return Self {
            discovery_thread: None,
            my_device,
            discovered_devices: HashMap::new(),
            receiver
        };
    }

    pub fn get_devices(&self) -> Option<Vec<DeviceInfo>> {
        let devices = self.receiver.try_recv().ok();

        if let Some(devices) = devices {
            return Some(devices.values().cloned().collect::<Vec<DeviceInfo>>());
        }

        return None;
    }

    pub fn stop(&self) {
    }
}