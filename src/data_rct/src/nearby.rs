use ble::BleAdvertisement;
use protocol::discovery::Device;
use crate::transmission::{DataTransmission, TransmissionSetupError};
use crate::transmission::tcp::TcpTransmissionListener;

pub struct NearbyServer {
    tcp_server: TcpTransmissionListener,
    ble_advertisement: BleAdvertisement
}

impl NearbyServer {
    pub fn new(my_device: Device) -> Result<Self, TransmissionSetupError> {
        let tcp_server = match TcpTransmissionListener::new() {
            Ok(result) => result,
            Err(_) => return Err(TransmissionSetupError::UnableToStartTcpServer)
        };

        let ble_advertisement = BleAdvertisement::new(my_device);

        return Ok(Self {
            tcp_server,
            ble_advertisement
        });
    }

    pub fn is_available(&self) -> bool {
        return self.ble_advertisement.is_powered_on();
    }

    pub fn advertise(&self) {
        self.ble_advertisement.start_advertising();
    }

    pub fn stop_advertising(&self) {
        self.ble_advertisement.stop_advertising();
    }
}
