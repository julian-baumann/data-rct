use std::fmt::Debug;
use std::{fs, thread};
use std::fs::File;
use std::io::{Read, Write};
use std::net::ToSocketAddrs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use gethostname::gethostname;
use local_ip_address::{local_ip};
use prost_stream::Stream;

use protocol::communication::{FileTransferIntent, TransferRequest, TransferRequestResponse};
use protocol::communication::transfer_request::Intent;
use protocol::discovery::{BluetoothLeConnectionInfo, Device, DeviceConnectionInfo, TcpConnectionInfo};

use crate::communication::{initiate_receiver_communication, initiate_sender_communication};
use crate::connection_request::ConnectionRequest;
use crate::convert_os_str;
use crate::discovery::Discovery;
use crate::encryption::{EncryptedReadWrite, EncryptedStream};
use crate::errors::ConnectErrors;
use crate::stream::{Close, NativeStreamDelegate};
use crate::transmission::tcp::{TcpClient, TcpServer};
use crate::transmission::TransmissionSetupError;

pub trait BleServerImplementationDelegate: Send + Sync + Debug {
    fn start_server(&self);
    fn stop_server(&self);
}

pub trait L2CAPDelegate: Send + Sync + Debug {
    fn open_l2cap_connection(&self, peripheral_uuid: String, psm: u32) -> Option<Box<dyn NativeStreamDelegate>>;
}

pub enum SendProgressState {
    Unknown,
    Connecting,
    Requesting,
    Transferring { progress: f64 },
    Finished,
    Declined
}

pub enum ConnectionIntentType {
    FileTransfer,
    Clipboard
}

pub trait ProgressDelegate: Send + Sync + Debug {
    fn progress_changed(&self, progress: SendProgressState);
}

pub trait NearbyConnectionDelegate: Send + Sync + Debug {
    fn received_connection_request(&self, request: Arc<ConnectionRequest>);
}

pub struct NearbyServer {
    pub device_connection_info: DeviceConnectionInfo,
    tcp_server: Option<TcpServer>,
    ble_server_implementation: Option<Box<dyn BleServerImplementationDelegate>>,
    ble_l2cap_client: Option<Box<dyn L2CAPDelegate>>,
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

    pub fn add_l2cap_client(&mut self, delegate: Box<dyn L2CAPDelegate>) {
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
            let tcp_server = TcpServer::new(self.nearby_connection_delegate.clone(), self.file_storage.clone()).await;
            if let Ok(tcp_server) = tcp_server {
                tcp_server.start_loop();

                let my_local_ip = local_ip().unwrap();

                println!("IP: {:?}", my_local_ip);
                println!("Port: {:?}", tcp_server.port);

                self.set_tcp_details(TcpConnectionInfo {
                    hostname: my_local_ip.to_string(),
                    port: tcp_server.port as u32,
                });

                self.tcp_server = Some(tcp_server);
            } else if let Err(error) = tcp_server {
                println!("Error trying to start TCP server: {:?}", error);
            }
        }

        self.advertise = true;

        if let Some(ble_advertisement_implementation) = &self.ble_server_implementation {
            ble_advertisement_implementation.start_server();
        };

        return Ok(());
    }

    async fn initiate_sender<T>(&self, raw_stream: T) -> Result<EncryptedStream<T>, ConnectErrors> where T: Read + Write + Close {
        return Ok(match initiate_sender_communication(raw_stream).await {
            Ok(stream) => stream,
            Err(error) => return Err(ConnectErrors::FailedToEncryptStream { error: error.to_string() })
        });
    }

    async fn connect(&self, device: Device) -> Result<Box<dyn EncryptedReadWrite>, ConnectErrors> {
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

        if let Ok(raw_stream) = tcp_stream {
            let encrypted_stream = self.initiate_sender(raw_stream).await?;
            return Ok(Box::new(encrypted_stream));
        }

        // Use BLE if TCP fails
        let Some(ble_connection_details) = &connection_details.ble else {
            return Err(ConnectErrors::FailedToGetBleDetails);
        };

        let Some(ble_l2cap_client) = &self.ble_l2cap_client else {
            return Err(ConnectErrors::InternalBleHandlerNotAvailable);
        };

        let connection = ble_l2cap_client.open_l2cap_connection(ble_connection_details.uuid.clone(), ble_connection_details.psm);

        let Some(connection) = connection else {
            return Err(ConnectErrors::FailedToEstablishBleConnection);
        };

        let encrypted_stream = self.initiate_sender(connection).await?;
        return Ok(Box::new(encrypted_stream));
    }

    fn update_progress(progress_delegate: &Option<Box<dyn ProgressDelegate>>, state: SendProgressState) {
        if let Some(progress_delegate) = progress_delegate {
            progress_delegate.progress_changed(state);
        }
    }

    pub async fn send_file(&self, receiver: Device, file_path: String, progress_delegate: Option<Box<dyn ProgressDelegate>>) -> Result<(), ConnectErrors> {
        NearbyServer::update_progress(&progress_delegate, SendProgressState::Connecting);

        let mut encrypted_stream = match self.connect(receiver).await {
            Ok(connection) => connection,
            Err(error) => return Err(error)
        };

        let mut proto_stream = Stream::new(&mut encrypted_stream);

        let path = Path::new(&file_path);
        let filename = path.file_name().expect("Failed to get file name");
        let metadata = fs::metadata(&file_path).expect("Failed to get metadata for file");
        let file_size = metadata.len();

        NearbyServer::update_progress(&progress_delegate, SendProgressState::Requesting);

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
            NearbyServer::update_progress(&progress_delegate, SendProgressState::Declined);
            return Err(ConnectErrors::Declined);
        }

        let mut file = File::open(file_path).expect("Failed to open file");
        let mut buffer = [0; 1024];

        NearbyServer::update_progress(&progress_delegate, SendProgressState::Transferring { progress: 0.0 });

        let mut all_read: usize = 0;

        while let Ok(read_size) = file.read(&mut buffer) {
            if read_size == 0 {
                break;
            }

            all_read += read_size;

            NearbyServer::update_progress(&progress_delegate, SendProgressState::Transferring { progress: (all_read as f64 / file_size as f64) });

            encrypted_stream.write_all(&buffer[..read_size])
                .expect("Failed to write file buffer");
        }

        NearbyServer::update_progress(&progress_delegate, SendProgressState::Finished);

        return Ok(());
    }

    pub fn handle_incoming_connection(&self, native_stream_handle: Box<dyn NativeStreamDelegate>) {
        let delegate = self.nearby_connection_delegate.clone();
        let file_storage = self.file_storage.clone();

        thread::spawn(move || {
            let connection_request = match initiate_receiver_communication(native_stream_handle, file_storage.clone()) {
                Ok(request) => request,
                Err(error) => {
                    println!("Encryption error {:}", error);
                    return;
                }
            };

            delegate.lock().expect("Failed to lock").received_connection_request(Arc::new(connection_request));
        });
    }

    pub fn stop(&mut self) {
        self.advertise = false;

        if let Some(ble_advertisement_implementation) = &self.ble_server_implementation {
            ble_advertisement_implementation.stop_server();
        }
    }
}
