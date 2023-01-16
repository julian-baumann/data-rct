use std::time::{Duration, Instant};
use data_rct::discovery::{DeviceInfo, Discovery};

const FOREIGN_DEVICE_ID: &str = "39FAC7A0-E581-4676-A9C5-0F6DC667567F";

fn get_my_device() -> DeviceInfo {
    return DeviceInfo {
        id: "B53CCB62-7DAB-4403-9FEB-F336834DB41F".to_string(),
        name: "Rust Device".to_string(),
        port: 42,
        device_type: "computer".to_string(),
        ip_address: "1.2.3.4".to_string()
    };
}

fn setup_foreign_discovery() -> Discovery {
    let discovery = Discovery::new(DeviceInfo {
        id: FOREIGN_DEVICE_ID.to_string(),
        name: "Discovery-Test Advertiser".to_string(),
        port: 52,
        device_type: "computer".to_string(),
        ip_address: "2.3.4.5".to_string()
    }).unwrap();

    discovery.advertise();

    return discovery;
}

#[test]
fn discovery() {
    let foreign_discovery = setup_foreign_discovery();

    let mut discovery = Discovery::new(get_my_device()).unwrap();
    discovery.start_search();

    let start = Instant::now();

    loop {
        let devices = discovery.get_devices();

        for device in devices {
            if device.id == FOREIGN_DEVICE_ID {
                discovery.stop_search();
                discovery.stop_advertising();
                discovery.stop().expect("Failed to stop discovery");
                foreign_discovery.stop().expect("Failed to stop foreign discovery");

                assert!(true);
                return;
            }
        }

        if start.elapsed() >= Duration::from_secs(20) {
            assert!(false, "No devices were found in 20s");
        }
    }
}