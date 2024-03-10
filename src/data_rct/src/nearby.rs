use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::net::ToSocketAddrs;
use std::path::Path;
use std::sync::Arc;
use std::{fs, thread};

use local_ip_address::local_ip;
use prost_stream::Stream;

use protocol::communication::transfer_request::Intent;
use protocol::communication::{FileTransferIntent, TransferRequest, TransferRequestResponse};
use protocol::discovery::{
    BluetoothLeConnectionInfo, Device, DeviceConnectionInfo, TcpConnectionInfo,
};
use tokio::sync::oneshot::{self, Sender};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::communication::{initiate_receiver_communication, initiate_sender_communication};
use crate::connection_request::ConnectionRequest;
use crate::discovery::Discovery;
use crate::encryption::{EncryptedReadWrite, EncryptedStream};
use crate::errors::ConnectErrors;
use crate::init_logger;
use crate::stream::NativeStreamDelegate;
use crate::transmission::tcp::{TcpClient, TcpServer};

pub trait BleServerImplementationDelegate: Send + Sync + Debug {
    fn start_server(&self);
    fn stop_server(&self);
}

pub trait L2CapDelegate: Send + Sync + Debug {
    fn open_l2cap_connection(&self, connection_id: String, peripheral_uuid: String, psm: u32);
}

pub enum ConnectionIntentType {
    FileTransfer,
    Clipboard,
}

pub enum SendProgressState {
    Unknown,
    Connecting,
    Requesting,
    Transferring { progress: f64 },
    Cancelled,
    Finished,
    Declined,
}

pub trait SendProgressDelegate: Send + Sync + Debug {
    fn progress_changed(&self, progress: SendProgressState);
}

pub trait NearbyConnectionDelegate: Send + Sync + Debug {
    fn received_connection_request(&self, request: Arc<ConnectionRequest>);
}

pub struct NearbyServerLockedVariables {
    pub device_connection_info: DeviceConnectionInfo,
    tcp_server: Option<TcpServer>,
    ble_server_implementation: Option<Box<dyn BleServerImplementationDelegate>>,
    ble_l2_cap_client: Option<Box<dyn L2CapDelegate>>,
    nearby_connection_delegate: Arc<std::sync::Mutex<Box<dyn NearbyConnectionDelegate>>>,
    pub advertise: bool,
    file_storage: String,
    l2cap_connections: HashMap<String, Sender<Box<dyn NativeStreamDelegate>>>,
}

pub struct NearbyServer {
    pub variables: Arc<RwLock<NearbyServerLockedVariables>>,
}

impl NearbyServer {
    pub fn new(
        my_device: Device,
        file_storage: String,
        delegate: Box<dyn NearbyConnectionDelegate>,
    ) -> Self {
        init_logger();

        let device_connection_info = DeviceConnectionInfo {
            device: Some(my_device.clone()),
            ble: None,
            tcp: None,
        };

        return Self {
            variables: Arc::new(RwLock::new(NearbyServerLockedVariables {
                device_connection_info,
                tcp_server: None,
                ble_server_implementation: None,
                ble_l2_cap_client: None,
                nearby_connection_delegate: Arc::new(std::sync::Mutex::new(delegate)),
                advertise: false,
                file_storage,
                l2cap_connections: HashMap::new(),
            })),
        };
    }

    pub fn add_l2_cap_client(&self, delegate: Box<dyn L2CapDelegate>) {
        self.variables.blocking_write().ble_l2_cap_client = Some(delegate);
    }

    pub fn add_bluetooth_implementation(
        &self,
        implementation: Box<dyn BleServerImplementationDelegate>,
    ) {
        self.variables.blocking_write().ble_server_implementation = Some(implementation)
    }

    pub fn change_device(&self, new_device: Device) {
        self.variables
            .blocking_write()
            .device_connection_info
            .device = Some(new_device);
    }

    pub fn set_bluetooth_le_details(&self, ble_info: BluetoothLeConnectionInfo) {
        self.variables.blocking_write().device_connection_info.ble = Some(ble_info)
    }

    pub fn set_tcp_details(&self, tcp_info: TcpConnectionInfo) {
        self.variables.blocking_write().device_connection_info.tcp = Some(tcp_info)
    }

    pub async fn start(&self) {
        if self.variables.read().await.tcp_server.is_none() {
            let delegate = self
                .variables
                .read()
                .await
                .nearby_connection_delegate
                .clone();
            let file_storage = self.variables.read().await.file_storage.clone();
            let tcp_server = TcpServer::new(delegate, file_storage).await;

            if let Ok(tcp_server) = tcp_server {
                let ip = local_ip();

                if let Ok(my_local_ip) = ip {
                    println!("IP: {:?}", my_local_ip);
                    println!("Port: {:?}", tcp_server.port);

                    tcp_server.start_loop();

                    self.set_tcp_details(TcpConnectionInfo {
                        hostname: my_local_ip.to_string(),
                        port: tcp_server.port as u32,
                    });

                    self.variables.write().await.tcp_server = Some(tcp_server);
                } else if let Err(error) = ip {
                    println!("Unable to obtain IP address: {:?}", error);
                }
            } else if let Err(error) = tcp_server {
                println!("Error trying to start TCP server: {:?}", error);
            }
        }

        self.variables.write().await.advertise = true;

        if let Some(ble_advertisement_implementation) =
            &self.variables.read().await.ble_server_implementation
        {
            ble_advertisement_implementation.start_server();
        };
    }

    async fn initiate_sender<T>(&self, raw_stream: T) -> Result<EncryptedStream<T>, ConnectErrors>
    where
        T: Read + Write,
    {
        initiate_sender_communication(raw_stream)
            .await
            .map_err(|err| ConnectErrors::FailedToEncryptStream {
                error: err.to_string(),
            })
    }

    pub fn handle_incoming_ble_connection(
        &self,
        connection_id: String,
        native_stream: Box<dyn NativeStreamDelegate>,
    ) {
        let sender = self
            .variables
            .blocking_write()
            .l2cap_connections
            .remove(&connection_id);

        if let Some(sender) = sender {
            let _ = sender.send(native_stream);
        }
    }

    async fn connect_tcp(
        &self,
        connection_details: &DeviceConnectionInfo,
    ) -> Result<Box<dyn EncryptedReadWrite>, ConnectErrors> {
        let Some(tcp_connection_details) = &connection_details.tcp else {
            return Err(ConnectErrors::FailedToGetTcpDetails);
        };

        let socket_string = format!(
            "{0}:{1}",
            tcp_connection_details.hostname, tcp_connection_details.port
        );
        println!("{:?}", socket_string);

        let mut socket_address = socket_string
            .to_socket_addrs()
            .map_err(|err| {
                println!("{err:?}");
                ConnectErrors::FailedToGetSocketAddress
            })?
            .as_slice()[0];
        socket_address.set_port(tcp_connection_details.port as u16);

        let tcp_stream =
            TcpClient::connect(socket_address).map_err(|_| ConnectErrors::FailedToOpenTcpStream)?;

        let encrypted_stream = self.initiate_sender(tcp_stream).await?;
        return Ok(Box::new(encrypted_stream));
    }

    async fn connect(&self, device: Device) -> Result<Box<dyn EncryptedReadWrite>, ConnectErrors> {
        let connection_details = Discovery::get_connection_details(device)
            .ok_or(ConnectErrors::FailedToGetConnectionDetails)?;

        let encrypted_stream = self
            .connect_tcp(&connection_details)
            .await
            .inspect_err(|err| println!("{err:?}"));
        if encrypted_stream.is_ok() {
            return encrypted_stream;
        }

        // Use BLE if TCP fails
        let ble_connection_details = &connection_details
            .ble
            .ok_or(ConnectErrors::FailedToGetBleDetails)?;

        let id = Uuid::new_v4().to_string();
        let (sender, receiver) = oneshot::channel::<Box<dyn NativeStreamDelegate>>();

        self.variables
            .write()
            .await
            .l2cap_connections
            .insert(id.clone(), sender);

        let ble_l2cap_client = &self.variables.read().await.ble_l2_cap_client;

        let ble_l2cap_client = ble_l2cap_client
            .as_ref()
            .ok_or(ConnectErrors::InternalBleHandlerNotAvailable)?;

        ble_l2cap_client.open_l2cap_connection(
            id.clone(),
            ble_connection_details.uuid.clone(),
            ble_connection_details.psm,
        );

        let connection = receiver
            .await
            .map_err(|_| ConnectErrors::FailedToEstablishBleConnection)?;

        let encrypted_stream = self.initiate_sender(connection).await?;
        return Ok(Box::new(encrypted_stream));
    }

    fn update_progress(
        progress_delegate: &Option<Box<dyn SendProgressDelegate>>,
        state: SendProgressState,
    ) {
        if let Some(progress_delegate) = progress_delegate {
            progress_delegate.progress_changed(state);
        }
    }

    pub async fn send_file(
        &self,
        receiver: Device,
        file_path: String,
        progress_delegate: Option<Box<dyn SendProgressDelegate>>,
    ) -> Result<(), ConnectErrors> {
        NearbyServer::update_progress(&progress_delegate, SendProgressState::Connecting);

        let mut encrypted_stream = self.connect(receiver).await?;

        let mut proto_stream = Stream::new(&mut encrypted_stream);

        let path = Path::new(&file_path);
        let filename = path.file_name().ok_or(ConnectErrors::FailedToGetFilename)?;
        let metadata =
            fs::metadata(&file_path).map_err(|err| ConnectErrors::FailedToGetFileMetadata {
                error: err.to_string(),
            })?;
        let file_size = metadata.len();

        NearbyServer::update_progress(&progress_delegate, SendProgressState::Requesting);

        let transfer_request = TransferRequest {
            device: self
                .variables
                .read()
                .await
                .device_connection_info
                .device
                .clone(),
            intent: Some(Intent::FileTransfer(FileTransferIntent {
                file_name: filename.to_str().map(|s| s.to_string()),
                file_size,
                multiple: false,
            })),
        };

        let _ = proto_stream.send(&transfer_request);

        let response = match proto_stream.recv::<TransferRequestResponse>() {
            Ok(message) => message,
            Err(error) => {
                return Err(ConnectErrors::FailedToGetTransferRequestResponse {
                    error: error.to_string(),
                })
            }
        };

        if !response.accepted {
            NearbyServer::update_progress(&progress_delegate, SendProgressState::Declined);
            return Err(ConnectErrors::Declined);
        }

        let mut file = OpenOptions::new()
            .write(false)
            .create(false)
            .read(true)
            .open(file_path)
            .expect("Failed to open file");

        let mut buffer = [0; 1024];

        NearbyServer::update_progress(
            &progress_delegate,
            SendProgressState::Transferring { progress: 0.0 },
        );

        let mut all_read: usize = 0;

        while let Ok(read_size) = file.read(&mut buffer) {
            if read_size == 0 {
                break;
            }

            all_read += read_size;

            NearbyServer::update_progress(
                &progress_delegate,
                SendProgressState::Transferring {
                    progress: (all_read as f64 / file_size as f64),
                },
            );

            encrypted_stream
                .write_all(&buffer[..read_size])
                .expect("Failed to write file buffer");
        }

        encrypted_stream.close();

        NearbyServer::update_progress(&progress_delegate, SendProgressState::Finished);
        return Ok(());
    }

    pub fn handle_incoming_connection(&self, native_stream_handle: Box<dyn NativeStreamDelegate>) {
        let delegate = self
            .variables
            .blocking_read()
            .nearby_connection_delegate
            .clone();
        let file_storage = self.variables.blocking_read().file_storage.clone();

        thread::spawn(move || {
            let mut encrypted_stream = match initiate_receiver_communication(native_stream_handle) {
                Ok(request) => request,
                Err(error) => {
                    println!("Encryption error {:}", error);
                    return;
                }
            };

            let mut prost_stream = Stream::new(&mut encrypted_stream);
            let Ok(transfer_request) = prost_stream
                .recv::<TransferRequest>()
                .inspect_err(|err| println!("Error {err:?}"))
            else {
                return;
            };

            let connection_request = ConnectionRequest::new(
                transfer_request,
                Box::new(encrypted_stream),
                file_storage.clone(),
            );

            delegate
                .lock()
                .expect("Failed to lock delegate")
                .received_connection_request(Arc::new(connection_request));
        });
    }

    pub fn stop(&self) {
        self.variables.blocking_write().advertise = false;

        if let Some(ble_advertisement_implementation) =
            &self.variables.blocking_read().ble_server_implementation
        {
            ble_advertisement_implementation.stop_server();
        }
    }
}
