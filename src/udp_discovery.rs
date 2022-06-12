use std::error::Error;
use std::net::{SocketAddr, UdpSocket};
use std::{thread};
use std::io::ErrorKind;
use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle};
use crate::transform::{ByteConvertable, get_utf8_message_part};

const DISCOVERY_PORTS: [u16; 3] = [42400, 42410, 42420];

#[derive(Clone)]
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

enum MessageType {
    Unknown,
    DeviceLookupRequest,
    DeviceInfo,
    RemoveDeviceFromDiscovery
}

pub struct UdpDiscovery {
    discovery_thread: Option<JoinHandle<()>>,
    pub my_device: DeviceInfo
}

impl UdpDiscovery {
    pub fn new(my_device: DeviceInfo) -> UdpDiscovery {

        return UdpDiscovery {
            discovery_thread: None,
            my_device
        };
    }

    pub fn start(&mut self, device_discovered: fn(DeviceInfo)) -> Result<(), Box<dyn Error>> {
        let device = self.my_device.clone();

        self.discovery_thread = Some(thread::spawn(move || {
            let _ = DiscoveryThread::new(device, device_discovered);
        }));

        return Ok(());
    }

    pub fn stop(&self) {
    }
}


struct DiscoveryThread {
    socket: Arc<Mutex<UdpSocket>>,
    my_device: DeviceInfo,
    discovered_device_callback: fn(DeviceInfo)
}

impl DiscoveryThread {
    pub fn new(my_device: DeviceInfo, callback: fn(DeviceInfo)) -> Result<DiscoveryThread, Box<dyn Error>> {
        let mut this = DiscoveryThread {
            socket: Arc::new(Mutex::new(DiscoveryThread::open_udp_socket()?)),
            my_device,
            discovered_device_callback: callback
        };

        DiscoveryThread::receive_loop(&mut this)?;

        return Ok(this);

    }

    fn receive_loop(&mut self) -> Result<(), Box<dyn Error>> {
        self.socket.try_lock().unwrap().set_broadcast(true)?;

        loop {
            let mut buf: [u8; 100] = [0; 100];
            let mut result;
            let sender_ip;

            let received = self.socket.lock().unwrap().recv_from(&mut buf);

            if let Ok(received) = received {
                result = Vec::from(&buf[0..received.0]);
                sender_ip = received.1;

                self.manage_request(sender_ip, &mut result);
            }
        }
    }

    fn open_udp_socket() -> Result<UdpSocket, Box<dyn Error>> {
        for port in DISCOVERY_PORTS {
            let socket = UdpSocket::bind("0.0.0.0:".to_owned() + &port.to_string());

            if let Ok(socket) = socket {
                return Ok(socket);
            }

            if let Err(socket) = socket {
                if socket.kind() != ErrorKind::AddrInUse {
                    return Err(socket)?;
                }
            }
        }

        return Err("All available ports are already used")?;
    }

    fn convert_message_type(input: &str) -> MessageType {
        return match input {
            "deviceLookupRequest" => MessageType::DeviceLookupRequest,
            "deviceInfo" => MessageType::DeviceInfo,
            "removeDeviceFromDiscovery" => MessageType::RemoveDeviceFromDiscovery,
            _ => MessageType::Unknown
        };
    }

    fn get_message_type_string(input: MessageType) -> String {
        return match input {
            MessageType::DeviceLookupRequest => "deviceLookupRequest".to_string(),
            MessageType::DeviceInfo => "deviceInfo".to_string(),
            MessageType::RemoveDeviceFromDiscovery => "removeDeviceFromDiscovery".to_string(),
            _ => "unknown".to_string()
        };
    }

    fn create_message_with_header(message_type: MessageType, body: &mut Vec<u8>) -> Vec<u8> {
        let mut message: Vec<u8> = Vec::new();
        message.append(&mut "1".as_bytes().to_vec());
        message.push(0u8);
        message.append(&mut DiscoveryThread::get_message_type_string(message_type).as_bytes().to_vec());
        message.push(0u8);
        message.append( body);

        return message;
    }

    fn answer_to_lookup_request(&self, message: &mut Vec<u8>, sender_ip: SocketAddr) {
        let sender_id = get_utf8_message_part(message);

        if let Some(sender_id) = sender_id {
            if sender_id.ne(&self.my_device.id) {
                let message: Vec<u8> = DiscoveryThread::create_message_with_header(MessageType::DeviceInfo, &mut self.my_device.to_bytes());

                self.socket.lock().unwrap().send_to(message.as_slice(), sender_ip).unwrap();
            }
        }
    }

    fn manage_request(&self, sender_ip: SocketAddr, message: &mut Vec<u8>) -> Option<()> {
        let version = get_utf8_message_part(message)?;

        if version == "1" {
            let message_type = get_utf8_message_part(message)?;

            let message_type = DiscoveryThread::convert_message_type(&message_type);

            match message_type {
                MessageType::DeviceLookupRequest => {
                    self.answer_to_lookup_request(message, sender_ip);
                },
                MessageType::DeviceInfo => {
                    let device = DeviceInfo::from_bytes(message, sender_ip.ip().to_string())?;
                    (self.discovered_device_callback)(device);
                }
                _ => return None
            }
        }

        return Some(());
    }
}