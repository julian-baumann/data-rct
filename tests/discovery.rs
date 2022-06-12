use data_rct::udp_discovery::{DeviceInfo, UdpDiscovery};

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
    let mut discovery = UdpDiscovery::new(get_my_device());
    let mut found_device = false;

    discovery.start(|device_info| {
        println!("discovered {}", device_info.name);
        // found_device = true;
    }).unwrap();

    loop {

    }
}