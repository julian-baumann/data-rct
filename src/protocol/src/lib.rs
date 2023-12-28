pub use prost;

pub mod discovery {
    include!(concat!(env!("OUT_DIR"), "/data_rct.discovery.rs"));
}

pub mod communication {
    include!(concat!(env!("OUT_DIR"), "/data_rct.communication.rs"));
}
