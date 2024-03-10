pub use prost;
use std::fmt::Debug;

pub mod discovery {
    include!(concat!(env!("OUT_DIR"), "/data_rct.discovery.rs"));
}

pub mod communication {
    include!(concat!(env!("OUT_DIR"), "/data_rct.communication.rs"));
}

pub trait DiscoveryDelegate: Send + Sync + Debug {
    fn device_added(&self, value: discovery::Device);
    fn device_removed(&self, device_id: String);
}
