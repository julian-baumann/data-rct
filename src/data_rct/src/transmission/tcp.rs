use std::{io, thread};
use std::net::SocketAddr;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use prost_stream::Stream;
use protocol::communication::TransferRequest;

use crate::communication::initiate_receiver_communication;
use crate::connection_request::ConnectionRequest;
use crate::nearby::NearbyConnectionDelegate;
use crate::stream::Close;

pub struct TcpServer {
    pub port: u16,
    listener: TcpListener,
    delegate: Arc<Mutex<Box<dyn NearbyConnectionDelegate>>>,
    file_storage: String
}

impl TcpServer {
    pub(crate) async fn new(delegate: Arc<Mutex<Box<dyn NearbyConnectionDelegate>>>, file_storage: String) -> Result<TcpServer, io::Error> {
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

                let mut encrypted_stream = match initiate_receiver_communication(tcp_stream) {
                    Ok(request) => request,
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

                let connection_request = ConnectionRequest::new(
                    transfer_request,
                    Box::new(encrypted_stream),
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
        let std_stream = std::net::TcpStream::connect_timeout(&address, Duration::from_secs(2))?;
        std_stream.set_nonblocking(false).expect("Failed to set non blocking");

        return Ok(std_stream);
    }
}

impl Close for TcpStream {
    fn close(&self) {
        // Do nothing. TCPStream closes automatically.
    }
}
