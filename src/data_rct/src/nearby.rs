use std::fmt::Debug;
use protocol::discovery::{Device};
use crate::transmission::{DataTransmission, TransmissionSetupError};
use crate::transmission::tcp::TcpTransmissionListener;

pub trait BleServerImplementationDelegate: Send + Sync + Debug {
    fn start_server(&self);
    fn stop_server(&self);
}

pub trait Connection {
    fn get_device_info(&self) -> Device;
    fn read(&self, length: u32) -> Vec<u8>;
    fn write(&self, data: Vec<u8>);
    fn disconnect(&self);
}

pub struct NearbyServer {
    pub my_device: Device,
    tcp_server: TcpTransmissionListener,
    ble_server_implementation: Option<Box<dyn BleServerImplementationDelegate>>,
    pub advertise: bool
}

impl NearbyServer {
    pub fn new(my_device: Device) -> Result<Self, TransmissionSetupError> {
        let tcp_server = match TcpTransmissionListener::new() {
            Ok(result) => result,
            Err(error) => return Err(TransmissionSetupError::UnableToStartTcpServer(error.to_string()))
        };

        return Ok(Self {
            my_device,
            tcp_server,
            ble_server_implementation: None,
            advertise: false
        });
    }

    pub fn add_bluetooth_implementation(&mut self, implementation: Box<dyn BleServerImplementationDelegate>) {
        self.ble_server_implementation = Some(implementation)
    }

    pub fn change_device(&mut self, new_device: Device) {
        self.my_device = new_device
    }

    pub fn start(&mut self) {
        self.advertise = true;

        if let Some(ble_advertisement_implementation) = &self.ble_server_implementation {
            ble_advertisement_implementation.start_server();
        }
    }

    pub fn get_tcp_port(&self) -> u16 {
        return self.tcp_server.port;
    }

    pub fn stop(&mut self) {
        self.advertise = false;

        if let Some(ble_advertisement_implementation) = &self.ble_server_implementation {
            ble_advertisement_implementation.stop_server();
        }
    }
}
