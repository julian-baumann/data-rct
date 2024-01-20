use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::net::ToSocketAddrs;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use gethostname::gethostname;
use local_ip_address::{list_afinet_netifas, local_ip};
use prost_stream::Stream;
use thiserror::Error;

use protocol::communication::{ClipboardTransferIntent, FileTransferIntent, TransferRequest, TransferRequestResponse};
use protocol::communication::transfer_request::Intent;
use protocol::discovery::{BluetoothLeConnectionInfo, Device, DeviceConnectionInfo, TcpConnectionInfo};

use crate::communication::{NativeStream, SenderConnection};
use crate::convert_os_str;
use crate::discovery::Discovery;
use crate::encryption::EncryptedStream;
use crate::transmission::tcp::{TcpClient, TcpServer};
use crate::transmission::TransmissionSetupError;

pub trait BleServerImplementationDelegate: Send + Sync + Debug {
    fn start_server(&self);
    fn stop_server(&self);
}

pub trait L2CAPClientDelegate: Send + Sync + Debug {
    fn open_l2cap_connection(&self, peripheral_uuid: String, psm: u32) -> Option<Arc<NativeStream>>;
}

pub struct ConnectionRequest {
    transfer_request: TransferRequest,
    connection: Arc<Mutex<Connection>>,
    file_storage: String,
}

impl ConnectionRequest {
    pub fn new(transfer_request: TransferRequest, connection: Connection, file_storage: String) -> Self {
        return Self {
            transfer_request,
            connection: Arc::new(Mutex::new(connection)),
            file_storage
        }
    }

    pub fn get_sender(&self) -> Device {
        return self.transfer_request.clone().device.expect("Device information missing");
    }

    pub fn get_intent(&self) -> Intent {
        return self.transfer_request.clone().intent.expect("Intent information missing");
    }

    pub fn decline(&self) {
        let mut connection_guard = self.connection.lock().unwrap(); // Handle the lock error appropriately
        let mut stream = match &mut *connection_guard {
            Connection::Tcp(encrypted_stream) => Stream::new(encrypted_stream),
        };

        let _ = stream.send(&TransferRequestResponse {
            accepted: false
        });
    }

    pub fn accept(&self) {
        let mut connection_guard = self.connection.lock().unwrap();

        let mut stream = match &mut *connection_guard {
            Connection::Tcp(encrypted_stream) => Stream::new(encrypted_stream),
        };

        let _ = stream.send(&TransferRequestResponse {
            accepted: true
        });

        match self.get_intent() {
            Intent::FileTransfer(file_transfer) => self.handle_file(connection_guard, file_transfer),
            Intent::Clipboard(clipboard) => self.handle_clipboard(clipboard)
        };
    }

    fn handle_clipboard(&self, clipboard_transfer_intent: ClipboardTransferIntent) {
        panic!("Not implemented yet");
    }

    fn handle_file(&self, mut connection_guard: MutexGuard<Connection>, file_transfer: FileTransferIntent) {
        let path = Path::new(&self.file_storage);
        let path = path.join(&file_transfer.file_name.unwrap_or_else(|| "temp.zip".to_string()));
        let path = path.into_os_string();

        println!("Creating file at {:?}", path);
        let mut file = File::create(path).expect("Failed to create file");

        let mut buffer = [0; 1024];

        println!("Locking connection");
        let stream = match &mut *connection_guard {
            Connection::Tcp(encrypted_stream) => encrypted_stream,
        };

        println!("Locked");

        while let Ok(read_size) = stream.read(&mut buffer) {
            if read_size == 0 {
                break;
            }

            file.write_all(&buffer[..read_size])
                .expect("Failed to write file to disk");
        }

        println!("written everything");
    }
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

    #[error("Failed to encrypt stream: {error}")]
    FailedToEncryptStream { error: String },

    #[error("Failed to determine file size: {error}")]
    FailedToDetermineFileSize { error: String },

    #[error("Failed to get transfer request response: {error}")]
    FailedToGetTransferRequestResponse { error: String },
}

pub enum Connection {
    Tcp(EncryptedStream<std::net::TcpStream>)
}

pub struct NearbyServer {
    pub device_connection_info: DeviceConnectionInfo,
    tcp_server: Option<TcpServer>,
    ble_server_implementation: Option<Box<dyn BleServerImplementationDelegate>>,
    ble_l2cap_client: Option<Box<dyn L2CAPClientDelegate>>,
    nearby_connection_delegate: Arc<Mutex<Box<dyn NearbyConnectionDelegate>>>,
    pub advertise: bool,
    file_storage: String
}

impl NearbyServer {
    pub fn new(my_device: Device, file_storage: String, delegate: Box<dyn NearbyConnectionDelegate>) -> Self {
        let device_connection_info = DeviceConnectionInfo {
            device: Some(my_device.clone()),
            ble: None,
            tcp: None
        };

        return Self {
            device_connection_info,
            tcp_server: None,
            ble_server_implementation: None,
            ble_l2cap_client: None,
            nearby_connection_delegate: Arc::new(Mutex::new(delegate)),
            advertise: false,
            file_storage
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
            let tcp_server = match TcpServer::new(self.nearby_connection_delegate.clone(), self.file_storage.clone()).await {
                Ok(result) => result,
                Err(error) => return Err(TransmissionSetupError::UnableToStartTcpServer { error: error.to_string() })
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

    async fn connect(&self, device: Device) -> Result<Connection, ConnectErrors> {
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

        let Ok(tcp_client) = tcp_stream else {
            println!("{:?}", tcp_stream.unwrap_err());
            return Err(ConnectErrors::FailedToOpenTcpStream);
        };

        let encrypted_stream = match SenderConnection::initiate_sender(tcp_client).await {
            Ok(stream) => stream,
            Err(error) => return Err(ConnectErrors::FailedToEncryptStream { error: error.to_string() })
        };

        return Ok(Connection::Tcp(encrypted_stream));
    }

    pub async fn send_file(&self, receiver: Device, file_path: String) -> Result<(), ConnectErrors> {
        let connection = match self.connect(receiver).await {
            Ok(connection) => connection,
            Err(error) => return Err(error)
        };

        let mut encrypted_stream = match connection {
            Connection::Tcp(encrypted_stream) => encrypted_stream
        };

        let mut proto_stream = Stream::new(&mut encrypted_stream);

        let path = Path::new(&file_path);
        let filename = path.file_name().expect("Failed to get file name");
        let metadata = fs::metadata(&file_path).expect("Failed to get metadata for file");
        let file_size = metadata.len();

        let transfer_request = TransferRequest {
            device: self.device_connection_info.device.clone(),
            intent: Some(Intent::FileTransfer(FileTransferIntent {
                file_name: convert_os_str(filename),
                file_size,
                multiple: false
            }))
        };

        let _ = proto_stream.send(&transfer_request);

        let response = match proto_stream.recv::<TransferRequestResponse>() {
            Ok(message) => message,
            Err(error) => return Err(ConnectErrors::FailedToGetTransferRequestResponse { error: error.to_string() })
        };

        if !response.accepted {
            return Err(ConnectErrors::Declined);
        }

        let mut file = File::open(file_path).expect("Failed to open file");
        let mut buffer = [0; 1024];

        // Read file and write it to the stream
        while let Ok(read_size) = file.read(&mut buffer) {
            if read_size == 0 {
                break;
            }

            encrypted_stream.write_all(&buffer[..read_size])
                .expect("Failed to write file buffer");
        }

        return Ok(());
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
