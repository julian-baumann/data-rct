use std::collections::HashMap;
use std::error::Error;
use std::net::{SocketAddr, UdpSocket};
use std::io::ErrorKind;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use crossbeam_channel::{Receiver};
use crate::discovery::{DeviceInfo, DiscoveryDelegate, ThreadCommunication};
use crate::transform::{ByteConvertable, get_utf8_message_part};

const DISCOVERY_PORTS: [u16; 4] = [42400, 42410, 42420, 42430];

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
        let port = get_utf8_message_part(message)?.parse::<u16>();
        let device_type = get_utf8_message_part(message)?;

        return match port {
            Ok(port) => {
                Some(DeviceInfo {
                    id,
                    name,
                    port,
                    device_type,
                    ip_address
                })
            }
            Err(_) => {
                None
            }
        }
    }
}

#[derive(Clone)]
enum MessageType {
    Unknown,
    DeviceLookupRequest,
    DeviceInfo,
    RemoveDeviceFromDiscovery
}

pub struct UdpDiscovery<> {
    socket: UdpSocket,
    my_device: DeviceInfo,
    communication_receiver: Receiver<ThreadCommunication>,
    advertise_my_device: bool,
    discovered_devices: Arc<RwLock<HashMap<String, DeviceInfo>>>,
    callback: Option<Arc<Mutex<Box<dyn DiscoveryDelegate>>>>
}

impl UdpDiscovery {
    pub fn new(my_device: DeviceInfo,
               communication_receiver: Receiver<ThreadCommunication>,
               discovered_devices: Arc<RwLock<HashMap<String, DeviceInfo>>>,
               callback: Option<Arc<Mutex<Box<dyn DiscoveryDelegate>>>>) -> Result<UdpDiscovery, Box<dyn Error>> {
        return Ok(UdpDiscovery {
            socket: UdpDiscovery::open_udp_socket()?,
            my_device,
            communication_receiver,
            advertise_my_device: false,
            discovered_devices,
            callback
        });
    }

    fn open_udp_socket() -> Result<UdpSocket, Box<dyn Error>> {
        for port in DISCOVERY_PORTS {
            let socket = UdpSocket::bind("0.0.0.0:".to_owned() + &port.to_string());

            if let Ok(socket) = socket {
                return Ok(socket);
            }

            if let Err(socket) = socket {
                let kind = socket.kind();
                if kind != ErrorKind::AddrInUse {
                    return Err(socket)?;
                }
            }
        }

        return Err("All available ports are already used")?;
    }

    fn send_lookup_signal(&self) {
        let mut result: Vec<u8> = Vec::new();
        result.append(&mut self.my_device.id.as_bytes().to_vec());
        result.push(0x00);

        let message: Vec<u8> = self.create_message_with_header(MessageType::DeviceLookupRequest, &mut result);

        for port in DISCOVERY_PORTS {
            if let Err(error) = self.socket.send_to(message.as_slice(), "255.255.255.255:".to_string() + port.to_string().as_str()) {
                println!("Failed to send lookup signal on port {port} ({error})");
            }
        }
    }

    pub fn start_loop(&mut self) -> Result<(), Box<dyn Error>> {
        self.socket.set_broadcast(true)?;
        self.socket.set_nonblocking(true)?;

        let mut start = Instant::now();
        let mut look_for_devices = false;
        let mut first_run = true;

        loop {
            let message = self.communication_receiver.try_recv();

            if let Ok(message) = message {
                match message {
                    ThreadCommunication::LookForDevices => { look_for_devices = true },
                    ThreadCommunication::StopLookingForDevices => { look_for_devices = false },
                    ThreadCommunication::AnswerToLookupRequest => { self.advertise_my_device = true },
                    ThreadCommunication::StopAnsweringToLookupRequest => { self.stop_advertising() },
                    ThreadCommunication::Shutdown => { return Ok(()) }
                }
            }

            if look_for_devices {
                if start.elapsed() >= Duration::from_secs(5) {
                    start = Instant::now();

                    self.send_lookup_signal();
                } else if first_run {
                    self.send_lookup_signal();
                }
            }

            let mut buf: [u8; 100] = [0; 100];
            let mut result;
            let sender_ip;

            let received = self.socket.recv_from(&mut buf);

            if let Ok(received) = received {
                result = Vec::from(&buf[0..received.0]);
                sender_ip = received.1;

                self.manage_request(sender_ip, &mut result);
            }

            first_run = false;
        }
    }

    fn convert_message_type(&self, input: &str) -> MessageType {
        return match input {
            "DeviceLookupRequest" => MessageType::DeviceLookupRequest,
            "DeviceInfo" => MessageType::DeviceInfo,
            "RemoveDeviceFromDiscovery" => MessageType::RemoveDeviceFromDiscovery,
            _ => MessageType::Unknown
        };
    }

    fn get_message_type_string(&self, input: MessageType) -> String {
        return match input {
            MessageType::DeviceLookupRequest => "DeviceLookupRequest".to_string(),
            MessageType::DeviceInfo => "DeviceInfo".to_string(),
            MessageType::RemoveDeviceFromDiscovery => "RemoveDeviceFromDiscovery".to_string(),
            _ => "unknown".to_string()
        };
    }

    fn create_message_with_header(&self, message_type: MessageType, body: &mut Vec<u8>) -> Vec<u8> {
        let mut message: Vec<u8> = Vec::new();
        message.append(&mut "1".as_bytes().to_vec());
        message.push(0u8);
        message.append(&mut self.get_message_type_string(message_type).as_bytes().to_vec());
        message.push(0u8);
        message.append( body);

        return message;
    }

    fn answer_to_lookup_request(&self, message: &mut Vec<u8>, sender_ip: SocketAddr) {
        let sender_id = get_utf8_message_part(message);

        if let Some(sender_id) = sender_id {
            if sender_id.ne(&self.my_device.id) {
                let message: Vec<u8> = self.create_message_with_header(MessageType::DeviceInfo, &mut self.my_device.to_bytes());

                self.socket.send_to(message.as_slice(), sender_ip).unwrap();
            }
        }
    }

    fn stop_advertising(&mut self) {
        self.advertise_my_device = false;
        let message: Vec<u8> = self.create_message_with_header(MessageType::RemoveDeviceFromDiscovery, &mut self.my_device.to_bytes());

        for port in DISCOVERY_PORTS {
            if let Err(error) = self.socket.send_to(message.as_slice(), "255.255.255.255:".to_string() + port.to_string().as_str()) {
                println!("Failed to send \"RemoveDeviceFromDiscovery\" signal on port {port} ({error})");
            }
        }
    }

    fn manage_request(&mut self, sender_ip: SocketAddr, message: &mut Vec<u8>) -> Option<()> {
        let version = get_utf8_message_part(message)?;

        if version == "1".to_string() {
            let message_type = get_utf8_message_part(message)?;

            let message_type = self.convert_message_type(&message_type);

            match message_type {
                MessageType::DeviceLookupRequest => {
                    if self.advertise_my_device {
                        self.answer_to_lookup_request(message, sender_ip);
                    }
                },
                MessageType::DeviceInfo => {
                    let device = DeviceInfo::from_bytes(message, sender_ip.ip().to_string())?;

                    let mut is_new_device = false;

                    if let Ok(mut discovered_devices) = self.discovered_devices.write() {
                        match discovered_devices.insert(device.id.clone(), device.clone()) {
                            Some(_) => is_new_device = false,
                            None => is_new_device = true
                        };
                    }

                    if is_new_device {
                        if let Some(callback) = &self.callback {
                            if let Ok(callback) = callback.lock() {
                                callback.device_added(device);
                            }
                        }
                    }
                },
                MessageType::RemoveDeviceFromDiscovery => {
                    let device_id = get_utf8_message_part(message)?;

                    let mut was_already_deleted = false;

                    if let Ok(mut discovered_devices) = self.discovered_devices.write() {
                        match discovered_devices.remove(&device_id) {
                            Some(_) => was_already_deleted = false,
                            None => was_already_deleted = true
                        }
                    }

                    if !was_already_deleted {
                        if let Some(callback) = &self.callback {
                            if let Ok(callback) = callback.lock() {
                                callback.device_removed(device_id);
                            }
                        }
                    }
                },
                _ => return None
            }
        }

        return Some(());
    }
}
