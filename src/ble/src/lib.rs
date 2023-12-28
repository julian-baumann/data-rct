mod platforms;
pub mod advertisement;

pub const DISCOVERY_SERVICE_UUID: &str = "68D60EB2-8AAA-4D72-8851-BD6D64E169B7";
pub const DISCOVERY_CHARACTERISTIC_UUID: &str = "0BEBF3FE-9A5E-4ED1-8157-76281B3F0DA5";


#[cfg(test)]
mod tests {
    use protocol::discovery::{Device};
    use protocol::discovery::device::DeviceType;
    use crate::advertisement::Advertisement;
    use crate::platforms::apple::Discovery;

    #[test]
    pub fn test_advertisement() {
        let my_device = Device {
            id: "43ED2550-3E5F-4ACC-BF58-DD0361A605C5".to_string(),
            name: "Test Device".to_string(),
            device_type: i32::from(DeviceType::Mobile),
            protocol_version: 1,
        };

        let advertisement = Advertisement::new(&my_device);
        advertisement.start_advertising();

        loop {}
    }

    #[test]
    pub fn test_discovery() {
        let peripheral = Discovery::new();
        while !peripheral.is_powered_on() {}

        peripheral.start_discovering_devices();

        while !peripheral.is_discovering() {}

        println!("Is advertising");

        loop {}
    }

}
