use std::fs::{self, File};
use std::fmt::Debug;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use prost_stream::Stream;
use protocol::communication::transfer_request::Intent;
use protocol::communication::{ClipboardTransferIntent, FileTransferIntent, TransferRequest, TransferRequestResponse};
use protocol::discovery::Device;
use tokio::sync::RwLock;
use crate::encryption::EncryptedReadWrite;
use crate::nearby::ConnectionIntentType;
use crate::BLE_BUFFER_SIZE;

pub enum ReceiveProgressState {
    Unknown,
    Handshake,
    Receiving { progress: f64 },
    Cancelled,
    Finished
}
pub trait ReceiveProgressDelegate: Send + Sync + Debug {
    fn progress_changed(&self, progress: ReceiveProgressState);
}

struct SharedVariables {
    receive_progress_delegate: Option<Box<dyn ReceiveProgressDelegate>>,
    should_cancel: bool
}

pub struct ConnectionRequest {
    transfer_request: TransferRequest,
    connection: Arc<Mutex<Box<dyn EncryptedReadWrite>>>,
    file_storage: String,
    variables: Arc<RwLock<SharedVariables>>
}

impl ConnectionRequest {
    pub fn new(transfer_request: TransferRequest, connection: Box<dyn EncryptedReadWrite>, file_storage: String) -> Self {
        return Self {
            transfer_request,
            connection: Arc::new(Mutex::new(connection)),
            file_storage,
            variables: Arc::new(RwLock::new(SharedVariables {
                receive_progress_delegate: None,
                should_cancel: false
            }))
        }
    }

    pub fn set_progress_delegate(&self, delegate: Box<dyn ReceiveProgressDelegate>) {
        self.variables.blocking_write().receive_progress_delegate = Some(delegate);
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

        connection_guard.close();
    }

    fn update_progress(&self, new_state: ReceiveProgressState) {
        if let Some(receive_progress_delegate) = &self.variables.blocking_read().receive_progress_delegate {
            receive_progress_delegate.progress_changed(new_state);
        }
    }

    pub async fn cancel(&self) {
        println!("trying to cancel");
        self.variables.write().await.should_cancel = true;
    }

    pub fn accept(&self) {
        self.update_progress(ReceiveProgressState::Handshake);
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

    fn handle_clipboard(&self, _clipboard_transfer_intent: ClipboardTransferIntent) {
        panic!("Not implemented yet");
    }

    fn handle_file(&self, mut stream: MutexGuard<Box<dyn EncryptedReadWrite>>, file_transfer: FileTransferIntent) {
        let path = Path::new(&self.file_storage);
        let path = path.join(&file_transfer.file_name.unwrap_or_else(|| "temp.zip".to_string()));
        let path = path.into_os_string();

        let mut file = File::create(path.clone()).expect("Failed to create file");

        let mut buffer = [0; BLE_BUFFER_SIZE];
        let mut all_read = 0.0;

        while let Ok(read_size) = stream.read(&mut buffer) {
            if self.variables.blocking_read().should_cancel {
                break;
            }

            if read_size <= 0 {
                break;
            }

            all_read += read_size as f64;

            file.write_all(&buffer[..read_size])
                .expect("Failed to write file to disk");

            let progress = all_read / file_transfer.file_size as f64;
            self.update_progress(ReceiveProgressState::Receiving { progress });

            if all_read == file_transfer.file_size as f64 {
                break;
            }
        }

        stream.close();

        if all_read < file_transfer.file_size as f64 {
            let _ = fs::remove_file(path);
            self.update_progress(ReceiveProgressState::Cancelled);
        } else {
            self.update_progress(ReceiveProgressState::Finished);
        }
    }
}
