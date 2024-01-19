use std::fmt::Debug;
use std::net::{ToSocketAddrs};
use std::sync::Arc;
use futures::channel::oneshot;
use gethostname::gethostname;
use local_ip_address::local_ip;
use thiserror::Error;
use tokio::io::{BufStream};
use protocol::discovery::{BluetoothLeConnectionInfo, Device, DeviceConnectionInfo, TcpConnectionInfo};
use crate::connection::{Connection, NativeStream};
use crate::discovery::Discovery;
use crate::transmission::{DataTransmission, TransmissionSetupError};
use crate::transmission::tcp::{TcpClient, TcpServer};

pub trait BleServerImplementationDelegate: Send + Sync + Debug {
    fn start_server(&self);
    fn stop_server(&self);
}

pub trait L2CAPClientDelegate: Send + Sync + Debug {
    fn open_l2cap_connection(&self, peripheral_uuid: String, psm: u32) -> Option<Arc<NativeStream>>;
}

pub struct ConnectionRequest {
}

pub trait NearbyConnectionDelegate: Send + Sync + Debug {
    fn received_connection_request(&self, request: Arc<ConnectionRequest>);
}

#[derive(Error, Debug)]
pub enum ConnectErrors {
    #[error("Peripheral is unreachable")]
    Unreachable,

    #[error("Failed to get connection details")]
    FailedToGetConnectionDetails,

    #[error("Peripheral declined the connection")]
    Declined,

    #[error("Failed to get tcp connection details")]
    FailedToGetTcpDetails,

    #[error("Failed to get socket address")]
    FailedToGetSocketAddress,

    #[error("Failed to open TCP stream")]
    FailedToOpenTcpStream,
}

pub struct NearbyServer {
    pub device_connection_info: DeviceConnectionInfo,
    tcp_server: Option<TcpServer>,
    ble_server_implementation: Option<Box<dyn BleServerImplementationDelegate>>,
    ble_l2cap_client: Option<Box<dyn L2CAPClientDelegate>>,
    nearby_connection_delegate: Box<dyn NearbyConnectionDelegate>,
    connection_oneshot: Option<oneshot::Sender<Arc<NativeStream>>>,
    pub advertise: bool
}

impl NearbyServer {
    pub fn new(my_device: Device, delegate: Box<dyn NearbyConnectionDelegate>) -> Self {
        let device_connection_info = DeviceConnectionInfo {
            device: Some(my_device),
            ble: None,
            tcp: None
        };

        return Self {
            device_connection_info,
            tcp_server: None,
            ble_server_implementation: None,
            ble_l2cap_client: None,
            nearby_connection_delegate: delegate,
            connection_oneshot: None,
            advertise: false
        };
    }

    pub fn add_l2cap_client(&mut self, delegate: Box<dyn L2CAPClientDelegate>) {
        self.ble_l2cap_client = Some(delegate);
    }

    pub fn add_bluetooth_implementation(&mut self, implementation: Box<dyn BleServerImplementationDelegate>) {
        self.ble_server_implementation = Some(implementation)
    }

    pub fn change_device(&mut self, new_device: Device) {
        self.device_connection_info.device = Some(new_device);
    }

    pub fn set_bluetooth_le_details(&mut self, ble_info: BluetoothLeConnectionInfo) {
        self.device_connection_info.ble = Some(ble_info)
    }

    pub fn set_tcp_details(&mut self, tcp_info: TcpConnectionInfo) {
        self.device_connection_info.tcp = Some(tcp_info)
    }

    pub async fn start(&mut self) -> Result<(), TransmissionSetupError> {
        if self.tcp_server.is_none() {
            let tcp_server = match TcpServer::new().await {
                Ok(result) => result,
                Err(error) => return Err(TransmissionSetupError::UnableToStartTcpServer(error.to_string()))
            };

            tcp_server.start_loop();
            let hostname = gethostname().into_string().expect("Failed to convert hostname to string");
            let my_local_ip = local_ip().unwrap();

            println!("Hostname: {:?}", hostname);
            println!("IP: {:?}", my_local_ip);
            println!("Port: {:?}", tcp_server.port);

            self.set_tcp_details(TcpConnectionInfo {
                hostname: my_local_ip.to_string(),
                port: tcp_server.port as u32,
            });

            self.tcp_server = Some(tcp_server);
        }

        self.advertise = true;

        if let Some(ble_advertisement_implementation) = &self.ble_server_implementation {
            ble_advertisement_implementation.start_server();
        };

        return Ok(());
    }

    // pub async fn accept(&self) {
    //     let Some(tcp_server) = &self.tcp_server else {
    //         return;
    //     };
    //
    //     println!("accept!");
    //     let Some(mut tcp_stream) = tcp_server.accept().await else {
    //         println!("accept error");
    //         return;
    //     };
    //
    //     println!("initiating receiver");
    //     let _ = Connection::initiate_receiver(&mut tcp_stream);
    // }

    pub async fn connect(&self, device: Device) -> Result<Connection, ConnectErrors> {
        let Some(connection_details) = Discovery::get_connection_details(device) else {
            return Err(ConnectErrors::FailedToGetConnectionDetails);
        };

        let Some(tcp_connection_details) = &connection_details.tcp else {
            return Err(ConnectErrors::FailedToGetTcpDetails);
        };

        let socket_string = format!("{0}:{1}", tcp_connection_details.hostname, tcp_connection_details.port);
        println!("{:?}", socket_string);

        let socket_address = socket_string.to_socket_addrs();

        let Ok(socket_address) = socket_address else {
            println!("{:?}", socket_address.unwrap_err());
            return Err(ConnectErrors::FailedToGetSocketAddress);
        };

        let mut socket_address = socket_address.as_slice()[0].clone();
        socket_address.set_port(tcp_connection_details.port as u16);

        let tcp_stream = TcpClient::connect(socket_address);

        let Ok(mut tcp_client) = tcp_stream else {
            println!("{:?}", tcp_stream.unwrap_err());
            return Err(ConnectErrors::FailedToOpenTcpStream);
        };

        Connection::initiate_sender(&mut tcp_client).await;

        return Err(ConnectErrors::Unreachable);
    }

    pub fn get_tcp_port(&self) -> Option<u16> {
        return match &self.tcp_server {
            None => None,
            Some(server) => Some(server.port)
        };
    }

    pub fn stop(&mut self) {
        self.advertise = false;

        if let Some(ble_advertisement_implementation) = &self.ble_server_implementation {
            ble_advertisement_implementation.stop_server();
        }
    }
}
