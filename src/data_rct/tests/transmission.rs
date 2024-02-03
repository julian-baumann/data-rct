use std::thread;
use data_rct::discovery::DeviceInfo;
use data_rct::stream::DeprecatedConnectStreamErrors;
use data_rct::transmission::Transmission;

#[test]
pub fn transmission_send() {
    let foreign_device: DeviceInfo = DeviceInfo {
        id: "B53CCB62-7DAB-4403-9FEB-F336834DB41F".to_string(),
        name: "Device 1".to_string(),
        port: 0,
        device_type: "computer".to_string(),
        ip_address: "127.0.0.1".to_string()
    };

    let my_device = DeviceInfo {
        id: "A689B035-B4AC-461F-8408-5CF1A5570592".to_string(),
        name: "Device 2".to_string(),
        port: 0,
        device_type: "computer".to_string(),
        ip_address: "127.0.0.1".to_string()
    };

    let receive_transmission = Transmission::new(foreign_device).unwrap();
    let foreign_device = receive_transmission.device_info.clone();

    thread::spawn(move || {
        loop {
            let request = receive_transmission.get_incoming_with_errors().unwrap().unwrap();
            request.accept().unwrap();
        }
    });

    let transmission = Transmission::new(my_device).unwrap();
    let _ = transmission.open(&foreign_device).unwrap();
}

#[test]
pub fn transmission_receive() {
    let my_device: DeviceInfo = DeviceInfo {
        id: "B53CCB62-7DAB-4403-9FEB-F336834DB41F".to_string(),
        name: "Device 1".to_string(),
        port: 0,
        device_type: "computer".to_string(),
        ip_address: "127.0.0.1".to_string()
    };

    let foreign_device = DeviceInfo {
        id: "A689B035-B4AC-461F-8408-5CF1A5570592".to_string(),
        name: "Device 2".to_string(),
        port: 0,
        device_type: "computer".to_string(),
        ip_address: "127.0.0.1".to_string()
    };

    let receive_transmission = Transmission::new(my_device.clone()).unwrap();

    let my_device_clone = receive_transmission.device_info.clone();
    let foreign_device_clone = foreign_device.clone();

    thread::spawn(move || {
        let transmission = Transmission::new(foreign_device_clone).unwrap();
        let _encrypted_stream = transmission.open(&my_device_clone).unwrap();
    });

    let transmission_request = receive_transmission.get_incoming_with_errors().unwrap().unwrap();
    assert_eq!(transmission_request.sender_id, foreign_device.id);
    assert_eq!(transmission_request.sender_name, foreign_device.name);
    assert!(transmission_request.uuid.len() > 0);

    transmission_request.accept().expect("Failed to accept transmission request");
}


#[test]
pub fn deny_transmission() {
    let foreign_device: DeviceInfo = DeviceInfo {
        id: "B53CCB62-7DAB-4403-9FEB-F336834DB41F".to_string(),
        name: "Device 1".to_string(),
        port: 0,
        device_type: "computer".to_string(),
        ip_address: "127.0.0.1".to_string()
    };

    let my_device = DeviceInfo {
        id: "A689B035-B4AC-461F-8408-5CF1A5570592".to_string(),
        name: "Device 2".to_string(),
        port: 0,
        device_type: "computer".to_string(),
        ip_address: "127.0.0.1".to_string()
    };

    let receive_transmission = Transmission::new(foreign_device).unwrap();
    let foreign_device = receive_transmission.device_info.clone();

    thread::spawn(move || {
        loop {
            let request = receive_transmission.get_incoming_with_errors().unwrap().unwrap();
            request.deny().unwrap();
        }
    });

    let transmission = Transmission::new(my_device).unwrap();
    let connection = transmission.open(&foreign_device);

    let is_expected_behaviour = match connection {
        Ok(_) => false,
        Err(error) => {
            match error {
                DeprecatedConnectStreamErrors::Rejected => true,
                _ => false
            }
        }
    };

    assert!(is_expected_behaviour);
}
