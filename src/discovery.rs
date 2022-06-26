use std::collections::HashMap;
use std::error::Error;
use std::thread;
use std::thread::JoinHandle;
use crate::observer::{IObserver, ISubject};
use crate::transform::{ByteConvertable, get_utf8_message_part};
use crate::udp_discovery::UdpDiscovery;

#[derive(Clone, PartialEq)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub port: u8,
    pub device_type: String,
    pub ip_address: String
}

impl ByteConvertable for DeviceInfo {
    fn to_bytes(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        result.append(&mut self.id.as_bytes().to_vec());
        result.push(0u8);
        result.append(&mut self.name.as_bytes().to_vec());
        result.push(0u8);
        result.append(&mut self.port.to_string().as_bytes().to_vec());
        result.push(0u8);
        result.append(&mut self.device_type.as_bytes().to_vec());
        result.push(0u8);

        return result;
    }

    fn from_bytes(message: &mut Vec<u8>, ip_address: String) -> Option<DeviceInfo> {
        let id = get_utf8_message_part(message)?;
        let name = get_utf8_message_part(message)?;
        let port = get_utf8_message_part(message)?;
        let device_type = get_utf8_message_part(message)?;

        let port = port.as_bytes().first()?.to_owned();

        return Some(DeviceInfo {
            id,
            name,
            port,
            device_type,
            ip_address
        });
    }
}

pub struct Discovery {
    discovery_thread: Option<JoinHandle<()>>,
    pub my_device: DeviceInfo,
    discovered_devices: HashMap<String, DeviceInfo>
}

impl IObserver for Discovery {
    fn update(&self) {
        todo!()
    }
}

impl Discovery {
    pub fn new(my_device: DeviceInfo) -> Discovery {
        return Discovery {
            discovery_thread: None,
            my_device,
            discovered_devices: HashMap::new()
        };
    }

    fn device_discovered(&self, device: DeviceInfo) {

    }

    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        let device = self.my_device.clone();
        let discovery = UdpDiscovery::new(device);
        discovery.unwrap().attach(self);

        self.discovery_thread = Some(thread::spawn(move || {
            let _ = discovery.unwrap().start_loop();
        }));

        return Ok(());
    }

    pub fn stop(&self) {
    }
}