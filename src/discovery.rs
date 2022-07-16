use std::collections::HashMap;
use std::error::Error;
use std::thread;
use std::thread::JoinHandle;
use crossbeam_channel::{Receiver, Sender};
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
    discovery_thread: JoinHandle<()>,
    pub my_device: DeviceInfo,
    discovered_devices: HashMap<String, DeviceInfo>,
    sender: Sender<ThreadCommunication>,
    discovery_receiver: Receiver<DiscoveryCommunication>,
}

#[derive(PartialEq)]
pub enum ThreadCommunication {
    LookForDevices,
    StopLookingForDevices,
    AnswerToLookupRequest,
    StopAnsweringToLookupRequest,
    Shutdown
}

pub enum DiscoveryCommunication {
    DeviceDiscovered(DeviceInfo),
    RemoveDevice(String)
}

impl Discovery {
    pub fn new(my_device: DeviceInfo) -> Discovery {
        let device = my_device.clone();
        let (sender, receiver) = crossbeam_channel::unbounded();
        let (discovery_sender, discovery_receiver) = crossbeam_channel::unbounded();

        let mut discovery = UdpDiscovery::new(device, discovery_sender.clone(), receiver.clone()).unwrap();

        let discovery_thread = thread::spawn(move || {
            discovery.start_loop().ok();
        });

        return Self {
            discovery_thread,
            my_device,
            discovered_devices: HashMap::new(),
            sender,
            discovery_receiver
        };
    }

    fn add_device(&mut self, device: DeviceInfo) {
        self.discovered_devices.insert(device.id.clone(), device);
    }

    fn remove_device(&mut self, device_id: &str) {
        self.discovered_devices.remove(device_id);
    }

    pub fn advertise(&self) {
        self.sender.send(ThreadCommunication::AnswerToLookupRequest).ok();
    }

    pub fn stop_advertising(&self) {
        self.sender.send(ThreadCommunication::StopAnsweringToLookupRequest).ok();
    }

    pub fn start_discovering(&self) {
        self.sender.send(ThreadCommunication::LookForDevices).ok();
    }

    pub fn get_devices(&mut self) -> Vec<DeviceInfo> {
        let new_devices = self.discovery_receiver.try_recv().ok();

        if let Some(new_devices) = new_devices {
            match new_devices {
                DiscoveryCommunication::DeviceDiscovered(device) => { self.add_device(device) }
                DiscoveryCommunication::RemoveDevice(device_id) => { self.remove_device(&device_id) }
            }
        }

        return self.discovered_devices.values().cloned().collect();
    }

    pub fn stop(self) -> Result<(), Box<dyn Error>> {
        self.sender.send(ThreadCommunication::Shutdown)?;
        self.discovery_thread.join().unwrap();

        return Ok(());
    }
}