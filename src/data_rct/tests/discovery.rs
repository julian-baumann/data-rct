use data_rct::discovery::{DeviceInfo, Discovery, DiscoveryMethod};
use std::time::{Duration, Instant};

const FOREIGN_DEVICE_ID: &str = "39FAC7A0-E581-4676-A9C5-0F6DC667567F";

fn get_my_device() -> DeviceInfo {
    return DeviceInfo {
        id: "B53CCB62-7DAB-4403-9FEB-F336834DB41F".to_string(),
        name: "Rust Device".to_string(),
        port: 42,
        device_type: "computer".to_string(),
        ip_address: "1.2.3.4".to_string(),
    };
}

fn setup_foreign_discovery(method: DiscoveryMethod) -> Discovery {
    let discovery = Discovery::new(
        DeviceInfo {
            id: FOREIGN_DEVICE_ID.to_string(),
            name: "Discovery-Test Advertiser".to_string(),
            port: 52,
            device_type: "computer".to_string(),
            ip_address: "2.3.4.5".to_string(),
        },
        method,
        None,
    )
    .unwrap();

    discovery.advertise();

    return discovery;
}

fn discover_device(method: DiscoveryMethod) {
    let discovery = Discovery::new(get_my_device(), method.clone(), None).unwrap();
    discovery.start_search();

    let start = Instant::now();

    loop {
        let devices = discovery.get_devices();

        for device in devices {
            if device.id == FOREIGN_DEVICE_ID {
                discovery.stop_search();
                discovery.stop_advertising();
                discovery.stop().expect("Failed to stop discovery");

                return;
            }
        }

        if start.elapsed() >= Duration::from_secs(20) {
            assert!(
                false,
                "No devices were found in 20s, mode: {}",
                method.to_string()
            );
        }
    }
}

#[test]
fn discovery() {
    let foreign_discovery = setup_foreign_discovery(DiscoveryMethod::Both);
    discover_device(DiscoveryMethod::Both);
    // sleep(Duration::from_secs(20));
    // loop {}
    foreign_discovery
        .stop()
        .expect("Failed to stop foreign discovery");
}
