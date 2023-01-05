mod udp;
mod mdns_sd;

use std::collections::HashMap;
use std::error::Error;
use std::thread;
use std::thread::JoinHandle;
use crossbeam_channel::{Receiver, Sender};
use crate::discovery::mdns_sd::MdnsSdDiscovery;
use crate::discovery::udp::UdpDiscovery;

#[derive(Clone, PartialEq)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub port: u16,
    pub device_type: String,
    pub ip_address: String
}

pub struct Discovery {
    pub my_device: DeviceInfo,
    discovery_thread: JoinHandle<()>,
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

trait PeripheralDiscovery {
    fn new(my_device: DeviceInfo,
           discovery_sender: Sender<DiscoveryCommunication>,
           communication_receiver: Receiver<ThreadCommunication>) -> Result<Self, Box<dyn Error>> where Self : Sized;
    fn start_loop(&mut self) -> Result<(), Box<dyn Error>>;
}

impl Discovery {
    pub fn new(my_device: DeviceInfo) -> Result<Discovery, Box<dyn Error>> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let (discovery_sender, discovery_receiver) = crossbeam_channel::unbounded();

        let mut udp_discovery = UdpDiscovery::new(
            my_device.clone(),
            discovery_sender.clone(),
            receiver.clone()
        )?;

        let mut mdns_discovery = MdnsSdDiscovery::new(
            my_device.clone(),
            discovery_sender.clone(),
            receiver.clone()
        )?;

        let discovery_thread = thread::spawn(move || {
            udp_discovery.start_loop().ok();
        });

        thread::spawn(move || {
            mdns_discovery.start_loop().ok();
        });

        return Ok(Self {
            discovery_thread,
            my_device,
            discovered_devices: HashMap::new(),
            sender,
            discovery_receiver
        });
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

    pub fn start_search(&self) {
        self.sender.send(ThreadCommunication::LookForDevices).ok();
    }

    pub fn stop_search(&self) {
        self.sender.send(ThreadCommunication::StopLookingForDevices).ok();
    }

    pub fn get_devices(&mut self) -> Vec<DeviceInfo> {
        let new_devices = self.discovery_receiver.try_recv();

        if let Ok(new_devices) = new_devices {
            match new_devices {
                DiscoveryCommunication::DeviceDiscovered(device) => { self.add_device(device) }
                DiscoveryCommunication::RemoveDevice(device_id) => { self.remove_device(&device_id) }
            }
        }

        return self.discovered_devices.values().cloned().collect();
    }

    pub fn stop(self) -> Result<(), Box<dyn Error>> {
        self.sender.send(ThreadCommunication::Shutdown)?;
        self.discovery_thread.join().ok();

        return Ok(());
    }
}