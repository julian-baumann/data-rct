extern crate core;

mod transform;
pub mod discovery;
pub mod transmission;
pub mod encryption;
pub mod stream;

const PROTOCOL_VERSION: u8 = 0x01;