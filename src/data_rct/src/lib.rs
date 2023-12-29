extern crate core;

mod transform;
pub mod discovery;
pub mod transmission;
pub mod encryption;
pub mod stream;
pub mod nearby;

pub use protocol::DiscoveryDelegate;
pub use protocol::discovery::{Device};
pub use protocol::communication::{SharedFileInfo, ClipboardTransferIntent};
use local_ip_address::local_ip;

const PROTOCOL_VERSION: u8 = 0x01;
const SERVICE_NAME: &str = "_data-rct._tcp.local.";

pub fn get_local_ip() -> String {
    return match local_ip() {
        Ok(ip) => ip.to_string(),
        Err(_) => String::new()
    };
}
