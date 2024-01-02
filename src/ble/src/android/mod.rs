use protocol::discovery::Device;

pub struct BleAdvertisement {

}

impl BleAdvertisement {
    pub fn new(device: Device) -> Self {
        Self {}
    }

    pub fn is_powered_on(&self) -> bool {
        return false;
    }

    pub fn start_advertising(&self) {
    }

    pub fn stop_advertising(&self) {
    }
}
