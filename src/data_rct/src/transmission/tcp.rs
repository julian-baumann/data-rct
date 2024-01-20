use std::error::Error;
use std::{io, thread};
use std::net::{SocketAddr};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use prost_stream::Stream;
use protocol::communication::TransferRequest;
use crate::communication::ReceiverConnection;
use crate::nearby::{Connection, ConnectionRequest, NearbyConnectionDelegate};

pub struct TcpServer {
    pub port: u16,
    listener: TcpListener,
    delegate: Arc<Mutex<Box<dyn NearbyConnectionDelegate>>>,
    file_storage: String
}

impl TcpServer {
    pub(crate) async fn new(delegate: Arc<Mutex<Box<dyn NearbyConnectionDelegate>>>, file_storage: String) -> Result<TcpServer, Box<dyn Error>> {
        let addresses = [
            SocketAddr::from(([0, 0, 0, 0], 80)),
            SocketAddr::from(([0, 0, 0, 0], 8080)),
            SocketAddr::from(([0, 0, 0, 0], 0))
        ];

        let listener = TcpListener::bind(&addresses[..])?;
        listener.set_nonblocking(false).expect("Failed to set non blocking");
        let port = listener.local_addr()?.port();

        return Ok(Self {
            port,
            listener,
            delegate,
            file_storage
        });
    }

    pub fn start_loop(&self) {
        let listener = self.listener.try_clone().expect("Failed to clone listener");
        let delegate = self.delegate.clone();
        let file_storage = self.file_storage.clone();

        thread::spawn(move || {
            loop {
                let Ok((tcp_stream, _socket_address)) = listener.accept() else {
                    continue
                };

                println!("initiating receiver");

                let mut encrypted_stream = match ReceiverConnection::initiate_receiver(tcp_stream) {
                    Ok(stream) => stream,
                    Err(error) => {
                        println!("Encryption error {:}", error);
                        continue;
                    }
                };

                let mut prost_stream = Stream::new(&mut encrypted_stream);
                let transfer_request = match prost_stream.recv::<TransferRequest>() {
                    Ok(message) => message,
                    Err(error) => {
                        println!("Error {:}", error);
                        continue;
                    }
                };

                println!("Received Transfer Request from {:}", transfer_request.clone().device.unwrap().name);

                let connection_request = ConnectionRequest::new(
                    transfer_request,
                    Connection::Tcp(encrypted_stream),
                    file_storage.clone()
                );

                delegate.lock().expect("Failed to lock").received_connection_request(Arc::new(connection_request));
            }
        });
    }
}

pub struct TcpClient {
}

impl TcpClient {
    pub fn connect(address: SocketAddr) -> Result<TcpStream, io::Error> {
        let std_stream = std::net::TcpStream::connect_timeout(&address, Duration::from_secs(5))?;
        std_stream.set_nonblocking(false).expect("Failed to set non blocking");
        // std_stream.set_nonblocking(true)?;
        // let stream = tokio::net::TcpStream::from_std(std_stream)?;

        return Ok(std_stream);
    }
}
