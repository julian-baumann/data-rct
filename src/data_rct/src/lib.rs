extern crate core;

pub mod discovery;
pub mod encryption;
pub mod stream;
pub mod nearby;
pub mod transmission;

pub use protocol;
pub use protocol::DiscoveryDelegate;
pub use protocol::discovery::{Device};
pub use protocol::communication::{SharedFileInfo, ClipboardTransferIntent};
use local_ip_address::local_ip;

const PROTOCOL_VERSION: u8 = 0x01;
const SERVICE_NAME: &str = "_data-rct._tcp.local.";
pub const BLE_SERVICE_UUID: & str = "68D60EB2-8AAA-4D72-8851-BD6D64E169B7";
pub const BLE_CHARACTERISTIC_UUID: &str = "0BEBF3FE-9A5E-4ED1-8157-76281B3F0DA5";


pub fn get_local_ip() -> String {
    return match local_ip() {
        Ok(ip) => ip.to_string(),
        Err(_) => String::new()
    };
}
