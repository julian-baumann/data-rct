use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use prost_stream::Stream;
use protocol::communication::transfer_request::Intent;
use protocol::communication::{ClipboardTransferIntent, FileTransferIntent, TransferRequest, TransferRequestResponse};
use protocol::discovery::Device;
use crate::encryption::EncryptedReadWrite;
use crate::nearby::{ConnectionIntentType};

pub struct ConnectionRequest {
    transfer_request: TransferRequest,
    connection: Arc<Mutex<Box<dyn EncryptedReadWrite>>>,
    file_storage: String,
}

impl ConnectionRequest {
    pub fn new(transfer_request: TransferRequest, connection: Box<dyn EncryptedReadWrite>, file_storage: String) -> Self {
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

    pub fn get_intent_type(&self) -> ConnectionIntentType {
        return match self.transfer_request.clone().intent.expect("Intent information missing") {
            Intent::FileTransfer(_) => ConnectionIntentType::FileTransfer,
            Intent::Clipboard(_) => ConnectionIntentType::FileTransfer
        };
    }

    pub fn get_file_transfer_intent(&self) -> Option<FileTransferIntent> {
        return match self.transfer_request.clone().intent.expect("Intent information missing") {
            Intent::FileTransfer(file_transfer_intent) => Some(file_transfer_intent),
            Intent::Clipboard(_) => None
        };
    }

    pub fn get_clipboard_intent(&self) -> Option<ClipboardTransferIntent> {
        return match self.transfer_request.clone().intent.expect("Intent information missing") {
            Intent::FileTransfer(_) => None,
            Intent::Clipboard(clipboard_intent) => Some(clipboard_intent)
        };
    }

    pub fn decline(&self) {
        let mut connection_guard = self.connection.lock().unwrap();
        let mut stream = Stream::new(&mut *connection_guard);

        let _ = stream.send(&TransferRequestResponse {
            accepted: false
        });
    }

    pub fn accept(&self) {
        let mut connection_guard = self.connection.lock().unwrap();
        let mut stream = Stream::new(&mut *connection_guard);

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

    fn handle_file(&self, mut stream: MutexGuard<Box<dyn EncryptedReadWrite>>, file_transfer: FileTransferIntent) {
        let path = Path::new(&self.file_storage);
        let path = path.join(&file_transfer.file_name.unwrap_or_else(|| "temp.zip".to_string()));
        let path = path.into_os_string();

        println!("Creating file at {:?}", path);
        let mut file = File::create(path).expect("Failed to create file");

        let mut buffer = [0; 1024];

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
