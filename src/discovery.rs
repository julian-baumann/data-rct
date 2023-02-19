mod udp;
mod mdns_sd;
use std::collections::HashMap;
use std::error::Error;
use std::{fmt, thread};
use std::fmt::Debug;
use std::sync::{Arc, Mutex, RwLock};
use crossbeam_channel::{Receiver, Sender};
use thiserror::Error;
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

#[derive(PartialEq, Clone, Copy)]
pub enum DiscoveryMethod {
    Both,
    MDNS,
    UDP
}

#[derive(Error, Debug)]
pub enum DiscoverySetupError {
    #[error("Unable to setup UDP Discovery")]
    UnableToSetupUdp,

    #[error("Unable to setup MDNS-SD Discovery")]
    UnableToSetupMdns
}

impl fmt::Display for DiscoveryMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DiscoveryMethod::Both => write!(f, "Both"),
            DiscoveryMethod::MDNS => write!(f, "MDNS"),
            DiscoveryMethod::UDP => write!(f, "UDP"),
        }
    }
}

trait PeripheralDiscovery {
    fn new(my_device: DeviceInfo,
           communication_receiver: Receiver<ThreadCommunication>,
           device_list: Arc<RwLock<HashMap<String, DeviceInfo>>>,
           callback: Option<Arc<Mutex<Box<dyn DiscoveryDelegate>>>>) -> Result<Self, Box<dyn Error>> where Self : Sized;
    fn start_loop(&mut self) -> Result<(), Box<dyn Error>>;
}

pub struct Discovery {
    pub my_device: DeviceInfo,
    discovered_devices: Arc<RwLock<HashMap<String, DeviceInfo>>>,
    sender: Sender<ThreadCommunication>
}

pub trait DiscoveryDelegate: Send + Sync + Debug {
    fn device_added(&self, value: DeviceInfo);
    fn device_removed(&self, device_id: String);
}

impl Discovery {
    pub fn new(my_device: DeviceInfo, method: DiscoveryMethod, delegate: Option<Box<dyn DiscoveryDelegate>>) -> Result<Discovery, DiscoverySetupError> {
        let (sender, receiver) = crossbeam_channel::unbounded();

        let discovered_devices = Arc::new(RwLock::new(HashMap::new()));
        let callback_arc = match delegate {
            Some(callback) => Some(Arc::new(Mutex::new(callback))),
            None => None
        };

        if method == DiscoveryMethod::UDP || method == DiscoveryMethod::Both {
            let udp_discovery = UdpDiscovery::new(
                my_device.clone(),
                receiver.clone(),
                Arc::clone(&discovered_devices),
                match &callback_arc {
                    Some(callback) => Some(Arc::clone(&callback)),
                    None => None
                }
            );

            if let Ok(mut udp_discovery) = udp_discovery {
                let builder = thread::Builder::new();
                let builder = builder.name("UDP Discovery".into());

                builder.spawn(move || {
                    if let Err(error) = udp_discovery.start_loop() {
                        println!("{}", error);
                    }
                }).ok();
            } else if let Err(error) = udp_discovery {
                println!("Error setting up UDP discovery \"{error}\"");
            }
        }

        if method == DiscoveryMethod::MDNS || method == DiscoveryMethod::Both {
            let mdns_discovery = MdnsSdDiscovery::new(
                my_device.clone(),
                receiver.clone(),
                Arc::clone(&discovered_devices),
                match &callback_arc {
                    Some(callback) => Some(Arc::clone(&callback)),
                    None => None
                }
            );

            if let Ok(mut mdns_discovery) = mdns_discovery {
                let builder = thread::Builder::new();
                let builder = builder.name("mDNS-SD Discovery".into());

                builder.spawn(move || {
                    if let Err(error) = mdns_discovery.start_loop() {
                        println!("{}", error);
                    }
                }).ok();
            } else if let Err(error) = mdns_discovery {
                println!("Error setting up MDNS-SD discovery \"{error}\"");
            }
        }

        return Ok(Self {
            my_device,
            discovered_devices,
            sender
        });
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

    pub fn get_devices(&self) -> Vec<DeviceInfo> {
        if let Ok(discovered_devices) = self.discovered_devices.read() {
            return discovered_devices.values().cloned().collect();
        }

        return Vec::new();
    }

    pub fn stop(self) -> Result<(), Box<dyn Error>> {
        self.stop_advertising();
        self.sender.send(ThreadCommunication::Shutdown)?;

        return Ok(());
    }
}
