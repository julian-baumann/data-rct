use std::sync::{Arc, Mutex};
use thiserror::Error;
use ble::discovery::BleDiscovery;
use protocol::DiscoveryDelegate;

#[derive(Error, Debug)]
pub enum DiscoverySetupError {
    #[error("Unable to setup UDP Discovery")]
    UnableToSetupUdp,

    #[error("Unable to setup MDNS-SD Discovery")]
    UnableToSetupMdns
}

pub struct Discovery {
    ble_discovery: BleDiscovery
}

impl Discovery {
    pub fn new(delegate: Option<Box<dyn DiscoveryDelegate>>) -> Result<Self, DiscoverySetupError> {
        let callback_arc = match delegate {
            Some(callback) => Some(Arc::new(Mutex::new(callback))),
            None => None
        };

        let ble_discovery = BleDiscovery::new(callback_arc);

        Ok(Self {
            ble_discovery
        })
    }

    pub fn start(&self) {
        self.ble_discovery.start();
    }

    pub fn stop(&self) {
        self.ble_discovery.stop();
    }

    // pub fn is_available(&self) -> bool {
    //     return self.ble_discovery.is_powered_on();
    // }

    // pub fn start(&self) {
    //     self.ble_discovery.start_advertising();
    // }
    //
    // pub fn stop(&self) {
    //     self.ble_discovery.stop_advertising();
    // }

    // pub fn get_devices(&self) -> Vec<Device> {
    //     return self.ble_discovery.get_devices();
    // }
}



// mod udp;
// mod mdns_sd;
//
// use std::collections::HashMap;
// use std::error::Error;
// use std::{fmt, thread};
// use std::fmt::Debug;
// use std::sync::{Arc, Mutex, RwLock};
// use crossbeam_channel::{Receiver, Sender};
// use thiserror::Error;
// use ble::platforms::apple::BleDiscovery;
// use protocol::discovery::Device;
// use protocol::DiscoveryDelegate;
// use crate::discovery::mdns_sd::MdnsSdDiscovery;
//
//
// #[derive(PartialEq)]
// pub enum ThreadCommunication {
//     StartDiscovery,
//     StopDiscovery,
//     StartAdvertising,
//     StopAdvertising,
//     Shutdown
// }
//
// pub enum DiscoveryCommunication {
//     DeviceDiscovered(Device),
//     RemoveDevice(String)
// }
//
// #[derive(PartialEq, Clone, Copy)]
// pub enum DiscoveryMethod {
//     Bonjour,
//     BLE,
//     Both
// }
//
// #[derive(Error, Debug)]
// pub enum DiscoverySetupError {
//     #[error("Unable to setup UDP Discovery")]
//     UnableToSetupUdp,
//
//     #[error("Unable to setup MDNS-SD Discovery")]
//     UnableToSetupMdns
// }
//
// impl fmt::Display for DiscoveryMethod {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             DiscoveryMethod::Both => write!(f, "Both"),
//             DiscoveryMethod::Bonjour => write!(f, "Bonjour"),
//             DiscoveryMethod::BLE => write!(f, "BLE")
//         }
//     }
// }
//
// trait PeripheralDiscovery {
//     fn new(my_device: Device,
//            communication_receiver: Receiver<ThreadCommunication>,
//            device_list: Arc<RwLock<HashMap<String, Device>>>,
//            callback: Option<Arc<Mutex<Box<dyn DiscoveryDelegate>>>>) -> Result<Self, Box<dyn Error>> where Self : Sized;
//     fn start_loop(&mut self) -> Result<(), Box<dyn Error>>;
// }
//
// pub struct Discovery {
//     pub my_device: Device,
//     discovered_devices: Arc<RwLock<HashMap<String, Device>>>,
//     sender: Sender<ThreadCommunication>
// }
//
// impl Discovery {
//     pub fn new(my_device: Device, method: DiscoveryMethod, delegate: Option<Box<dyn DiscoveryDelegate>>) -> Result<Discovery, DiscoverySetupError> {
//         let (sender, receiver) = crossbeam_channel::unbounded();
//
//         let discovered_devices = Arc::new(RwLock::new(HashMap::new()));
//         let callback_arc = match delegate {
//             Some(callback) => Some(Arc::new(Mutex::new(callback))),
//             None => None
//         };
//
//         if method == DiscoveryMethod::BLE || method == DiscoveryMethod::Both {
//             let udp_discovery = BleDiscovery::new(
//                 my_device.clone(),
//                 receiver.clone(),
//                 Arc::clone(&discovered_devices),
//                 match &callback_arc {
//                     Some(callback) => Some(Arc::clone(&callback)),
//                     None => None
//                 }
//             );
//
//             if let Ok(mut udp_discovery) = udp_discovery {
//                 let builder = thread::Builder::new();
//                 let builder = builder.name("UDP Discovery".into());
//
//                 builder.spawn(move || {
//                     if let Err(error) = udp_discovery.start_loop() {
//                         println!("{}", error);
//                     }
//                 }).ok();
//             } else if let Err(error) = udp_discovery {
//                 println!("Error setting up UDP discovery \"{error}\"");
//             }
//         }
//
//         if method == DiscoveryMethod::Bonjour || method == DiscoveryMethod::Both {
//             let mdns_discovery = MdnsSdDiscovery::new(
//                 my_device.clone(),
//                 receiver.clone(),
//                 Arc::clone(&discovered_devices),
//                 match &callback_arc {
//                     Some(callback) => Some(Arc::clone(&callback)),
//                     None => None
//                 }
//             );
//
//             if let Ok(mut mdns_discovery) = mdns_discovery {
//                 let builder = thread::Builder::new();
//                 let builder = builder.name("mDNS-SD Discovery".into());
//
//                 builder.spawn(move || {
//                     if let Err(error) = mdns_discovery.start_loop() {
//                         println!("{}", error);
//                     }
//                 }).ok();
//             } else if let Err(error) = mdns_discovery {
//                 println!("Error setting up MDNS-SD discovery \"{error}\"");
//             }
//         }
//
//         return Ok(Self {
//             my_device,
//             discovered_devices,
//             sender
//         });
//     }
//
//     pub fn advertise(&self) {
//         self.sender.send(ThreadCommunication::StartAdvertising).ok();
//     }
//
//     pub fn stop_advertising(&self) {
//         self.sender.send(ThreadCommunication::StopAdvertising).ok();
//     }
//
//     pub fn start_search(&self) {
//         self.sender.send(ThreadCommunication::StartDiscovery).ok();
//     }
//
//     pub fn stop_search(&self) {
//         self.sender.send(ThreadCommunication::StopDiscovery).ok();
//     }
//
//     pub fn get_devices(&self) -> Vec<Device> {
//         if let Ok(discovered_devices) = self.discovered_devices.read() {
//             return discovered_devices.values().cloned().collect();
//         }
//
//         return Vec::new();
//     }
//
//     pub fn stop(self) -> Result<(), Box<dyn Error>> {
//         self.stop_advertising();
//         self.sender.send(ThreadCommunication::Shutdown)?;
//
//         return Ok(());
//     }
// }
