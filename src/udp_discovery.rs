use std::error::Error;
use std::net::{SocketAddr, UdpSocket};
use std::io::ErrorKind;
use crate::discovery::DeviceInfo;
use crate::observer::{IObserver, ISubject};
use crate::transform::{ByteConvertable, get_utf8_message_part};

const DISCOVERY_PORTS: [u16; 3] = [42400, 42410, 42420];

enum MessageType {
    Unknown,
    DeviceLookupRequest,
    DeviceInfo,
    RemoveDeviceFromDiscovery
}

pub struct UdpDiscovery<'a, T: IObserver> {
    socket: UdpSocket,
    my_device: DeviceInfo,
    device_observers: Vec<&'a T>
}

impl<'a, T: IObserver> ISubject<'a, T> for UdpDiscovery<'a, T> {
    fn attach(&mut self, observer: &'a T) {
        self.device_observers.push(observer);
    }
    fn detach(&mut self, observer: &'a T) {
        // if let Some(idx) = self.device_observers.iter().position(|x| *x == observer) {
        //     self.device_observers.remove(idx);
        // }
    }

    fn notify_observers(&self) {
        for item in self.device_observers.iter() {
            item.update();
        }
    }
}

impl<'a, T: IObserver> UdpDiscovery<'a, T> {
    pub fn new(my_device: DeviceInfo) -> Result<UdpDiscovery<'a, T>, Box<dyn Error>> {
        return Ok(UdpDiscovery {
            socket: UdpDiscovery::<'a, T>::open_udp_socket()?,
            my_device,
            device_observers: Vec::new()
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

    pub fn start_loop(&self) -> Result<(), Box<dyn Error>> {
        self.socket.set_broadcast(true)?;
        self.socket.set_nonblocking(true)?;

        // loop {
        //     let mut buf: [u8; 100] = [0; 100];
        //     let mut result;
        //     let sender_ip;
        //
        //     let received = self.socket.recv_from(&mut buf);
        //
        //     if let Ok(received) = received {
        //         result = Vec::from(&buf[0..received.0]);
        //         sender_ip = received.1;
        //
        //         self.manage_request(sender_ip, &mut result);
        //     }
        // }
        let mut buf: [u8; 100] = [0; 100];
        let mut result;
        let sender_ip;

        let received = self.socket.recv_from(&mut buf);

        // match received {
        //     Ok(num_bytes) => {
        //         println!("I received {} bytes!", num_bytes.0)
        //     },
        //     Err(ref err) if err.kind() != ErrorKind::WouldBlock => {
        //         println!("Something went wrong: {}", err)
        //     }
        //     _ => {}
        // }

        if let Ok(received) = received {
            result = Vec::from(&buf[0..received.0]);
            sender_ip = received.1;

            self.manage_request(sender_ip, &mut result);
        }

        return Ok(());
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

    fn manage_request(&self, sender_ip: SocketAddr, message: &mut Vec<u8>) -> Option<()> {
        let version = get_utf8_message_part(message)?;

        if version == "1".to_string() {
            let message_type = get_utf8_message_part(message)?;

            let message_type = self.convert_message_type(&message_type);

            match message_type {
                MessageType::DeviceLookupRequest => {
                    self.answer_to_lookup_request(message, sender_ip);
                },
                MessageType::DeviceInfo => {
                    let device = DeviceInfo::from_bytes(message, sender_ip.ip().to_string())?;
                    self.notify_observers();
                }
                _ => return None
            }
        }

        return Some(());
    }
}