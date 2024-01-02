use std::collections::{HashMap};
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use crossbeam_channel::{Receiver, Sender};
use tokio::runtime::{Runtime};
use tokio::time::{sleep, timeout};
use uuid::Uuid;
use protocol::discovery::{Device, DeviceDiscoveryMessage};
use protocol::discovery::device_discovery_message::DeviceData;
use protocol::DiscoveryDelegate;
use protocol::prost::Message;
use crate::{DISCOVERY_CHARACTERISTIC_UUID, DISCOVERY_SERVICE_UUID};

enum DiscoveryState {
    Stop
}

struct DiscoveredDevice {
    device: Device,
    last_update: Instant
}

pub struct BleDiscovery {
    async_runtime: Runtime,
    central_adapter: Adapter,
    discovered_devices: Arc<RwLock<HashMap<String, DiscoveredDevice>>>,
    discovery_delegate: Option<Arc<Mutex<Box<dyn DiscoveryDelegate>>>>,
    sender: Sender<DiscoveryState>,
    receiver: Receiver<DiscoveryState>
}

impl BleDiscovery {
    pub fn new(discovery_delegate: Option<Arc<Mutex<Box<dyn DiscoveryDelegate>>>>) -> Self {
        let async_runtime = tokio::runtime::Builder::new_multi_thread()
            .max_blocking_threads(1)
            .enable_time()
            .build()
            .expect("Failed to initiate multi-thread tokio runtime.");

        let manager = async_runtime.block_on(Manager::new()).expect("Failed to create new Manager.");

        let central = async_runtime.block_on(manager.adapters())
            .expect("Unable to fetch adapter list.")
            .into_iter()
            .nth(0)
            .expect("Unable to find adapters.");

        let discovered_devices = Arc::new(RwLock::new(HashMap::new()));

        let (sender, receiver) = crossbeam_channel::unbounded();

        return Self {
            async_runtime,
            central_adapter: central,
            discovered_devices,
            discovery_delegate,
            sender,
            receiver
        };
    }

    fn handle_device_data(data: Vec<u8>, discovered_devices: Arc<RwLock<HashMap<String, DiscoveredDevice>>>, discovery_delegate: Option<Arc<Mutex<Box<dyn DiscoveryDelegate>>>>) {
        let discovery_message = DeviceDiscoveryMessage::decode_length_delimited(data.as_slice());

        let Ok(discovery_message) = discovery_message else {
            return;
        };

        match discovery_message.device_data {
            None => {}
            Some(DeviceData::Device(device)) => {
                if let Ok(mut discovered_devices) = discovered_devices.write() {
                    let already_discovered = discovered_devices.insert(device.id.clone(), DiscoveredDevice {
                        device: device.clone(),
                        last_update: Instant::now()
                    });

                    if !already_discovered.is_none() {
                        return;
                    }

                    match discovery_delegate {
                        None => {}
                        Some(delegate) => {
                            if let Ok(delegate) = delegate.lock() {
                                delegate.device_added(device)
                            }
                        }
                    }
                };
            }
            Some(DeviceData::DeviceId(device_id)) => {
                if let Ok(mut discovered_devices) = discovered_devices.write() {
                    discovered_devices.remove(device_id.as_str());

                    match discovery_delegate {
                        None => {}
                        Some(delegate) => {
                            if let Ok(delegate) = delegate.lock() {
                                delegate.device_removed(device_id)
                            }
                        }
                    }
                };
            }
        }
    }

    pub fn stop(&self) {
        let _ = self.sender.send(DiscoveryState::Stop);
    }

    pub fn start(&self) {
        let central = self.central_adapter.clone();
        let discovered_devices = self.discovered_devices.clone();
        let discovery_delegate = self.discovery_delegate.clone();
        let receiver = self.receiver.clone();

        self.async_runtime.spawn(async move {
            let scan_filter = ScanFilter {
                services: vec![
                    Uuid::from_str(DISCOVERY_SERVICE_UUID).expect("Failed to parse discovery service UUID")
                ]
            };

            central.start_scan(scan_filter)
                .await
                .expect("Failed to start scanning.");

            loop {
                if let Ok(event) = receiver.try_recv() {
                    match event {
                        DiscoveryState::Stop => {
                            break;
                        }
                    }
                }

                for (id, device) in discovered_devices.read().expect("Failed to lock discovered_devices.").iter() {
                    if device.last_update.elapsed() >= Duration::from_secs(5) {
                        discovered_devices
                            .write()
                            .expect("Failed to lock discovered_devices for write access")
                            .remove(id);

                        if let Some(discovery_delegate) = discovery_delegate.clone() {
                            discovery_delegate
                                .lock()
                                .expect("Failed to lock discovery_delegate")
                                .device_removed(id.to_string());
                        }
                    }
                }

                let Ok(peripherals) = central.peripherals().await else {
                    println!("Error while trying to get peripherals");
                    continue;
                };

                for peripheral in peripherals.iter() {
                    println!("checking connection...");
                    let Ok(is_connected) = timeout(Duration::from_secs(1), peripheral.is_connected()).await else {
                        println!("Timeout! is_connected 1");
                        continue;
                    };

                    println!("checked connection");

                    let Ok(is_connected) = is_connected else {
                        println!("Failed to connect to peripheral");
                        continue;
                    };

                    println!("is Connected {:}", is_connected);

                    if !is_connected {
                        let connection_successful = timeout(Duration::from_secs(1), peripheral.connect()).await;

                        let Ok(connection_successful) = connection_successful else {
                            println!("Timeout trying to connect!");
                            continue;
                        };

                        let Ok(_) = connection_successful else {
                            println!("Connection unsuccessful");
                            continue;
                        };
                    }

                    println!("discovering services...");
                    let _ = peripheral.discover_services();
                    println!("discovered services");

                    println!("get characteristics...");
                    let characteristics = peripheral.characteristics();
                    println!("got characteristics");

                    let discovery_characteristic = characteristics
                        .iter()
                        .find(|c| c.uuid == Uuid::parse_str(DISCOVERY_CHARACTERISTIC_UUID).expect("Failed to parse UUID"));

                    let Some(discovery_characteristic) = discovery_characteristic else {
                        println!("Couldn't find characteristic");
                        continue;
                    };

                    println!("pre read");
                    let discovery_data = timeout(Duration::from_secs(1), peripheral.read(&discovery_characteristic)).await;
                    println!("after read");

                    let Ok(discovery_data) = discovery_data else {
                        println!("read timeout");
                        continue;
                    };

                    println!("disconnecting...");
                    let _ = timeout(Duration::from_secs(1), peripheral.disconnect()).await;
                    println!("disconnected");

                    let Ok(discovery_data) = discovery_data else {
                        println!("read error");
                        continue;
                    };

                    println!("handle data...");
                    BleDiscovery::handle_device_data(discovery_data, discovered_devices.clone(), discovery_delegate.clone());
                    println!("handled data");
                }

                println!("sleeping");
                sleep(Duration::from_millis(500)).await;
                println!("slept");
            }
        });
    }
}
