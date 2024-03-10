#[cfg(target_os = "android")]
use android_logger::Config;
#[cfg(target_os = "android")]
use log::LevelFilter;

pub use protocol;
pub use protocol::communication::ClipboardTransferIntent;
pub use protocol::discovery::Device;
pub use protocol::DiscoveryDelegate;

pub mod communication;
pub mod connection_request;
pub mod discovery;
pub mod encryption;
pub mod errors;
pub mod nearby;
pub mod stream;
pub mod transmission;

pub const BLE_SERVICE_UUID: &str = "68D60EB2-8AAA-4D72-8851-BD6D64E169B7";
pub const BLE_CHARACTERISTIC_UUID: &str = "0BEBF3FE-9A5E-4ED1-8157-76281B3F0DA5";

#[cfg(target_os = "android")]
pub fn init_logger() {
    android_logger::init_once(Config::default().with_max_level(LevelFilter::Trace));
}

#[cfg(not(target_os = "android"))]
pub fn init_logger() {
    // Do nothing
}
