use std::str::FromStr;
use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::api::bleuuid::BleUuid;
use btleplug::platform::{Adapter, Manager};
use futures::StreamExt;
use tokio::runtime::Runtime;
use uuid::Uuid;
use protocol::discovery::Device;
use protocol::prost::Message;
use crate::{DISCOVERY_CHARACTERISTIC_UUID, DISCOVERY_SERVICE_UUID};

pub struct Advertisement {
    async_runtime: Runtime,
    central_adapter: Adapter,
    my_device: Vec<u8>
}

impl Advertisement {
    pub fn new(my_device: &Device) -> Self {
        let async_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
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

    pub fn start_advertising(&self) {
        let central = self.central_adapter.clone();
        let device_data = self.my_device.clone();

        self.async_runtime.spawn(async move {
            let mut events = central
                .events()
                .await
                .expect("Failed to get events.");

            let scan_filter = ScanFilter {
                services: vec![
                    Uuid::from_str(DISCOVERY_SERVICE_UUID).expect("Failed to parse discovery service UUID")
                ]
            };

            central
                .start_scan(scan_filter)
                .await
                .expect("Failed to start scanning.");

            while let Some(event) = events.next().await {
                match event {
                    CentralEvent::DeviceDiscovered(id) => {
                        println!("DeviceDiscovered: {:?}", id);
                        let peripheral = central.peripheral(&id).await;

                        if let Ok(peripheral) = peripheral {
                            let _ = peripheral.connect();
                        }
                    }
                    CentralEvent::DeviceConnected(id) => {
                        println!("DeviceConnected: {:?}", id);
                        let peripheral = central.peripheral(&id).await;

                        if let Ok(peripheral) = peripheral {
                            let result = peripheral.discover_services().await;

                            if let Ok(()) = result {
                                let characteristics = peripheral.characteristics();

                                let discovery_characteristic = characteristics
                                    .iter()
                                    .find(|c| c.uuid == Uuid::parse_str(DISCOVERY_CHARACTERISTIC_UUID).unwrap());

                                if let Some(discovery_characteristic) = discovery_characteristic {
                                    let _ = peripheral
                                        .write(&discovery_characteristic, &device_data, WriteType::WithoutResponse)
                                        .await;
                                }
                            }
                        }
                    }
                    CentralEvent::DeviceDisconnected(id) => {
                        println!("DeviceDisconnected: {:?}", id);
                    }
                    CentralEvent::ServiceDataAdvertisement { id, service_data } => {
                        println!("ServiceDataAdvertisement: {:?}, {:?}", id, service_data);
                    }
                    CentralEvent::ServicesAdvertisement { id, services } => {
                        let services: Vec<String> = services.into_iter().map(|s| s.to_short_string()).collect();
                        println!("ServicesAdvertisement: {:?}, {:?}", id, services);
                    }
                    _ => {}
                }
            }
        });
    }
}
