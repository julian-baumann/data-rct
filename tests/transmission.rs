use std::io::Cursor;
use data_rct::discovery::DeviceInfo;
use data_rct::transmission::{Stream, Transmission};

// #[test]
// pub fn transmission() {
//     let my_device = DeviceInfo {
//         id: "B53CCB62-7DAB-4403-9FEB-F336834DB41F".to_string(),
//         name: "Rust Device".to_string(),
//         port: 42,
//         device_type: "computer".to_string(),
//         ip_address: "1.2.3.4".to_string()
//     };
//
//     let transmission = Transmission::new(&my_device);
//
//     if let Ok(transmission) = transmission {
//         if let Some(stream) = transmission.accept() {
//
//         }
//     }
// }

// #[test]
// pub fn send() {
//     let my_device = DeviceInfo {
//         id: "B53CCB62-7DAB-4403-9FEB-F336834DB41F".to_string(),
//         name: "Rust Device".to_string(),
//         port: 42,
//         device_type: "computer".to_string(),
//         ip_address: "1.2.3.4".to_string()
//     };
//
//     let memory_stream: Cursor<Vec<u8>> = Cursor::new(Vec::new());
//     let transmission = Transmission::new(&my_device).unwrap();
//     // transmission.connect(Box::new(memory_stream), &my_device);
// }

#[test]
pub fn transmission() {
    let my_device = DeviceInfo {
        id: "B53CCB62-7DAB-4403-9FEB-F336834DB41F".to_string(),
        name: "Rust Device".to_string(),
        port: 42,
        device_type: "computer".to_string(),
        ip_address: "1.2.3.4".to_string()
    };

    let transmission = Transmission::new(&my_device);
}