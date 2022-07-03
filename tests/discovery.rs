use std::thread;
use std::time::Duration;
use data_rct::discovery::{DeviceInfo, Discovery};

fn get_my_device() -> DeviceInfo {
    return DeviceInfo {
        id: "B53CCB62-7DAB-4403-9FEB-F336834DB41F".to_string(),
        name: "Rust Device".to_string(),
        port: 42,
        device_type: "computer".to_string(),
        ip_address: "1.2.3.4".to_string()
    };
}

#[test]
fn start_discovery() {
    let discovery = Discovery::new(get_my_device());

    loop {
        thread::sleep(Duration::from_millis(2000));
        let devices = discovery.get_devices();

        if let Some(devices) = devices {
            for device in devices {
                println!("Found {}", device.name);
            }
        } else {
            println!("Found nothing");
        }
    }
}