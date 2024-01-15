use std::error::Error;
use std::{io, thread};
use std::net::{SocketAddr};
use std::thread::Thread;
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use crate::connection::Connection;


pub struct TcpServer {
    pub port: u16,
    listener: TcpListener
}

impl TcpServer {
    pub(crate) async fn new() -> Result<TcpServer, Box<dyn Error>> {
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
            listener
        });
    }

    pub fn start_loop(&self) {
        let listener = self.listener.try_clone().expect("Failed to clone listener");

        thread::spawn(move || {
            loop {
                let Ok((tcp_stream, _socket_address)) = listener.accept() else {
                    continue
                };

                println!("initiating receiver");
                let _ = Connection::initiate_receiver(tcp_stream);
            }
        });
    }

    // pub fn accept(&self) -> Option<TcpStream> {
    //     if let Ok((tcp_stream, _socket_address)) = self.listener.accept().await {
    //         return Some(tcp_stream);
    //     }
    //
    //     return None;
    // }
}

pub struct TcpClient {
    // stream: TcpStream
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

// impl Read for TcpClient {
//     fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>  {
//         self.listener.read(buf)
//     }
// }
//
// impl Write for TcpClient {
//     fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
//         self.listener.write(buf)
//     }
//
//     fn flush(&mut self) -> io::Result<()> {
//         self.listener.flush()
//     }
// }
