use std::cell::RefCell;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Adapter, Manager};
use futures::StreamExt;
use tokio::runtime::Runtime;
use uuid::Uuid;
use protocol::discovery::Device;
use protocol::prost::Message;
use crate::{DISCOVERY_CHARACTERISTIC_UUID, DISCOVERY_SERVICE_UUID};

pub struct BleAdvertisement {
    async_runtime: Runtime,
    central_adapter: Adapter,
    my_device: Vec<u8>
}

impl BleAdvertisement {
    pub fn new(my_device: &Device) -> Self {
        let async_runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();

        let manager = async_runtime.block_on(Manager::new()).unwrap();

        let central = async_runtime.block_on(manager.adapters())
            .expect("Unable to fetch adapter list.")
            .into_iter()
            .nth(0)
            .expect("Unable to find adapters.");

        let device_data = my_device.encode_length_delimited_to_vec();

        return Self {
            async_runtime,
            central_adapter: central,
            my_device: device_data
        };
    }

    pub fn change_device(&mut self, device: &Device) {
        self.my_device = device.encode_length_delimited_to_vec();
    }

    pub fn stop_advertising(&self) {
        let _ = self.central_adapter.stop_scan();
    }

    pub fn start_advertising(&self) {
        println!("starting advertisement");

        let central = self.central_adapter.clone();
        let device_data = self.my_device.clone();

        let test = self.async_runtime.block_on(async move {
            let mut events = central.events()
                .await
                .expect("Failed to get events.");

            let scan_filter = ScanFilter {
                services: vec![
                    Uuid::from_str(DISCOVERY_SERVICE_UUID).expect("Failed to parse discovery service UUID")
                ]
            };

            central.start_scan(scan_filter)
                .await
                .expect("Failed to start scanning.");

            println!("Started scan");

            while let Some(event) = events.next().await {
                match event {
                    CentralEvent::DeviceDiscovered(id) => {
                        println!("DeviceDiscovered: {:?}", id);
                        let peripheral = central.peripheral(&id).await;

                        if let Ok(peripheral) = peripheral {
                            let _ = peripheral.connect().await;
                        }
                    }
                    CentralEvent::DeviceConnected(id) => {
                        println!("DeviceConnected: {:?}", id);
                        let peripheral = central.peripheral(&id).await;

                        let Ok(peripheral) = peripheral else {
                            return;
                        };

                        let Ok(()) = peripheral.discover_services().await else {
                            return;
                        };

                        let characteristics = peripheral.characteristics();

                        let discovery_characteristic = characteristics
                            .iter()
                            .find(|c| c.uuid == Uuid::parse_str(DISCOVERY_CHARACTERISTIC_UUID).unwrap());

                        if let Some(discovery_characteristic) = discovery_characteristic {
                            let _ = peripheral
                                .write(&discovery_characteristic, &device_data, WriteType::WithoutResponse)
                                .await;
                        }

                        let _ = peripheral.disconnect().await;
                    }
                    _ => {}
                }
            }
        });
    }
}
