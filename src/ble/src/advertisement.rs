use std::collections::HashSet;
use std::str::FromStr;
use std::time::Duration;
use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Adapter, Manager, Peripheral, PeripheralId};
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use futures::StreamExt;
use tokio::runtime::{Runtime};
use tokio::time::sleep;
use uuid::Uuid;
use protocol::discovery::{Device, device_discovery_message, DeviceDiscoveryMessage};
use protocol::discovery::device_discovery_message::DeviceData;
use protocol::prost::Message;
use crate::{DISCOVERY_CHARACTERISTIC_UUID, DISCOVERY_SERVICE_UUID};

#[derive(Clone)]
enum BleAction {
    Advertise(Vec<u8>),
    StopAdvertising(Vec<u8>)
}

pub struct BleAdvertisement {
    async_runtime: Runtime,
    central_adapter: Adapter,
    my_device: Device,
    thread_communication_sender: Sender<BleAction>,
    thread_communication_receiver: Receiver<BleAction>
}

impl BleAdvertisement {
    pub fn new(my_device: &Device) -> Self {
        let async_runtime = tokio::runtime::Builder::new_multi_thread()
            .build()
            .unwrap();

        let manager = async_runtime.block_on(Manager::new()).unwrap();

        let central = async_runtime.block_on(manager.adapters())
            .expect("Unable to fetch adapter list.")
            .into_iter()
            .nth(0)
            .expect("Unable to find adapters.");

        let (sender, receiver) = crossbeam_channel::unbounded();

        BleAdvertisement::start_ble_event_thread(&async_runtime, central.clone(), receiver.clone());

        return Self {
            async_runtime,
            central_adapter: central,
            my_device: my_device.clone(),
            thread_communication_sender: sender,
            thread_communication_receiver: receiver
        };
    }

    fn start_ble_event_thread(async_runtime: &Runtime, central: Adapter, receiver: Receiver<BleAction>) {
        async_runtime.spawn(async move {
            let mut action: BleAction;
            let mut peripheral_ids: HashSet<PeripheralId> = HashSet::new();

            loop {
                let message = receiver.try_recv();

                if let Ok(message) = message {
                    action = message;
                    break;
                }
            }

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

            while let Some(event) = events.next().await {
                match event {
                    CentralEvent::DeviceDiscovered(id) => {
                        let peripheral = central.peripheral(&id).await;

                        if let Ok(peripheral) = peripheral {
                            let _ = peripheral.connect().await;
                        }
                    }
                    CentralEvent::DeviceConnected(id) => {
                        let central = central.clone();
                        let mut action = action.clone();
                        let receiver = receiver.clone();

                        let already_exists = !peripheral_ids.insert(id.clone());

                        if already_exists {
                            continue;
                        }

                        tokio::spawn(async move {
                            loop {
                                let peripheral = central.peripheral(&id).await;

                                let Ok(peripheral) = peripheral else {
                                    continue;
                                };

                                let Ok(()) = peripheral.discover_services().await else {
                                    continue;
                                };

                                let characteristics = peripheral.characteristics();

                                let discovery_characteristic = characteristics
                                    .iter()
                                    .find(|c| c.uuid == Uuid::parse_str(DISCOVERY_CHARACTERISTIC_UUID).unwrap());

                                let message = receiver.try_recv();

                                if let Ok(message) = message {
                                    action = message;
                                }

                                let data = match action {
                                    BleAction::Advertise(ref data) => Some(data),
                                    BleAction::StopAdvertising(ref remove_data) => Some(remove_data)
                                };

                                if let (Some(discovery_characteristic), Some(data)) = (discovery_characteristic, data) {
                                    let _ = peripheral
                                        .write(&discovery_characteristic, data, WriteType::WithoutResponse)
                                        .await;
                                }

                                let _ = peripheral.disconnect().await;
                            }
                        });
                    }
                    _ => {}
                }
            }
        });
    }

    pub fn change_device(&mut self, device: &Device) {
        self.my_device = device.clone();
    }

    pub fn start_advertising(&self) {
        let add_device = DeviceDiscoveryMessage {
            device_data: Some(DeviceData::Device(self.my_device.clone()))
        }.encode_length_delimited_to_vec();

        let _ = self.thread_communication_sender.send(BleAction::Advertise(add_device));
    }

    pub fn stop_advertising(&self) {
        let remove_device = DeviceDiscoveryMessage {
            device_data: Some(DeviceData::DeviceId(self.my_device.id.clone()))
        }.encode_length_delimited_to_vec();

        let _ = self.thread_communication_sender.send(BleAction::StopAdvertising(remove_device));
    }
}
