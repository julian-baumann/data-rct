use std::error::Error;
use std::io;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use crate::transmission::{DataTransmission, Stream};

pub struct TcpTransmissionListener {
    pub port: u16,
    listener: TcpListener
}

impl Stream for TcpStream {}

impl DataTransmission for TcpTransmissionListener {
    fn new() -> Result<TcpTransmissionListener, Box<dyn Error>> {
        let addresses = [
            SocketAddr::from(([0, 0, 0, 0], 42420)),
            SocketAddr::from(([0, 0, 0, 0], 0))
        ];

        let listener = TcpListener::bind(&addresses[..])?;
        let port = listener.local_addr()?.port();

        return Ok(Self {
            port,
            listener
        });
    }

    fn accept(&self) -> Option<Box<dyn Stream>> {
        if let Ok((tcp_stream, _socket_address)) = self.listener.accept() {
            return Some(Box::new(tcp_stream));
        }

        return None;
    }
}


// listener

impl Stream for TcpTransmissionClient {}

pub struct TcpTransmissionClient {
    listener: TcpStream
}

impl TcpTransmissionClient {
    pub fn connect(address: SocketAddr) -> Result<TcpTransmissionClient, Box<dyn Error>> {
        let listener = TcpStream::connect(address)?;

        return Ok(Self {
            listener
        });
    }
}

impl Read for TcpTransmissionClient {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>  {
        self.listener.read(buf)
    }
}

impl Write for TcpTransmissionClient {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.listener.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.listener.flush()
    }
}