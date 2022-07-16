use std::error::Error;
use std::net::{SocketAddr, UdpSocket};
use std::io::ErrorKind;
use std::time::{Duration, Instant};
use crossbeam_channel::{Receiver, Sender};
use crate::discovery::{DeviceInfo, DiscoveryCommunication, ThreadCommunication};
use crate::transform::{ByteConvertable, get_utf8_message_part};

const DISCOVERY_PORTS: [u16; 3] = [42400, 42410, 42420];

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

pub struct UdpDiscovery<> {
    socket: UdpSocket,
    my_device: DeviceInfo,
    discovery_sender: Sender<DiscoveryCommunication>,
    communication_receiver: Receiver<ThreadCommunication>,
    advertise_my_device: bool
}

impl UdpDiscovery {
    pub fn new(my_device: DeviceInfo, discovery_sender: Sender<DiscoveryCommunication>, communication_receiver: Receiver<ThreadCommunication>) -> Result<UdpDiscovery, Box<dyn Error>> {
        return Ok(UdpDiscovery {
            socket: UdpDiscovery::open_udp_socket()?,
            my_device,
            discovery_sender,
            communication_receiver,
            advertise_my_device: false
        });
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

    fn send_lookup_signal(&self) {
        let mut result: Vec<u8> = Vec::new();
        result.append(&mut self.my_device.id.as_bytes().to_vec());
        result.push(0x00);

        let message: Vec<u8> = self.create_message_with_header(MessageType::DeviceLookupRequest, &mut result);

        for port in DISCOVERY_PORTS {
            self.socket.send_to(message.as_slice(), "255.255.255.255:".to_string() + port.to_string().as_str()).unwrap();
        }
    }

    pub fn start_loop(&mut self) -> Result<(), Box<dyn Error>> {
        self.socket.set_broadcast(true)?;
        self.socket.set_nonblocking(true)?;

        let mut start = Instant::now();
        let mut look_for_devices = false;

        loop {
            let message = self.communication_receiver.try_recv();

            if let Ok(message) = message {
                match message {
                    ThreadCommunication::LookForDevices => { look_for_devices = true },
                    ThreadCommunication::StopLookingForDevices => { look_for_devices = false },
                    ThreadCommunication::AnswerToLookupRequest => { self.advertise_my_device = true },
                    ThreadCommunication::StopAnsweringToLookupRequest => { self.advertise_my_device = false },
                    ThreadCommunication::Shutdown => {
                        return Ok(())
                    }
                }
            }

            if look_for_devices {
                if start.elapsed() >= Duration::from_secs(5) {
                    start = Instant::now();

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
        }
    }

    fn convert_message_type(&self, input: &str) -> MessageType {
        return match input {
            "deviceLookupRequest" => MessageType::DeviceLookupRequest,
            "deviceInfo" => MessageType::DeviceInfo,
            "removeDeviceFromDiscovery" => MessageType::RemoveDeviceFromDiscovery,
            _ => MessageType::Unknown
        };
    }

    fn get_message_type_string(&self, input: MessageType) -> String {
        return match input {
            MessageType::DeviceLookupRequest => "deviceLookupRequest".to_string(),
            MessageType::DeviceInfo => "deviceInfo".to_string(),
            MessageType::RemoveDeviceFromDiscovery => "removeDeviceFromDiscovery".to_string(),
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
                    self.discovery_sender.try_send(DiscoveryCommunication::DeviceDiscovered(device)).ok();
                },
                MessageType::RemoveDeviceFromDiscovery => {
                    let device_id = get_utf8_message_part(message)?;
                    self.discovery_sender.try_send(DiscoveryCommunication::RemoveDevice(device_id)).ok();
                },
                _ => return None
            }
        }

        return Some(());
    }
}