use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex, RwLock};
use crossbeam_channel::{Receiver};
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use crate::discovery::{DeviceInfo, DiscoveryDelegate, PeripheralDiscovery, ThreadCommunication};
use crate::PROTOCOL_VERSION;

pub struct MdnsSdDiscovery {
    mdns_daemon: ServiceDaemon,
    my_device: DeviceInfo,
    communication_receiver: Receiver<ThreadCommunication>,
    discovered_devices: Arc<RwLock<HashMap<String, DeviceInfo>>>,
    delegate: Option<Arc<Mutex<Box<dyn DiscoveryDelegate>>>>
}

const SERVICE_NAME: &str = "_data-rct._tcp.local.";

impl MdnsSdDiscovery {
    fn advertise(&self) {
        let mut properties = HashMap::new();
        properties.insert("deviceId".to_string(), self.my_device.id.to_string());
        properties.insert("deviceName".to_string(), self.my_device.name.to_string());
        properties.insert("protocolVersion".to_string(), PROTOCOL_VERSION.to_string());
        properties.insert("type".to_string(), self.my_device.device_type.to_string());
        properties.insert("port".to_string(), self.my_device.port.to_string());

        let my_device = ServiceInfo::new(
            SERVICE_NAME,
            &self.my_device.name,
            &(self.my_device.name.replace(" ", "-").to_string() + ".local."),
            &self.my_device.ip_address,
            self.my_device.port,
            Some(properties)
        );

        if let Ok(my_device) = my_device {
            self.mdns_daemon.register(my_device).ok();
        }
    }

    fn stop_advertising(&self) {
        self.mdns_daemon.unregister(&(self.my_device.name.replace(" ", "-").to_string() + ".local.")).ok();
    }
}

impl PeripheralDiscovery for MdnsSdDiscovery {
    fn new(my_device: DeviceInfo,
           communication_receiver: Receiver<ThreadCommunication>,
           discovered_devices: Arc<RwLock<HashMap<String, DeviceInfo>>>,
           delegate: Option<Arc<Mutex<Box<dyn DiscoveryDelegate>>>>) -> Result<MdnsSdDiscovery, Box<dyn Error>> {
        return Ok(MdnsSdDiscovery {
            mdns_daemon: ServiceDaemon::new()?,
            my_device,
            communication_receiver,
            discovered_devices,
            delegate
        });
    }

    fn start_loop(&mut self) -> Result<(), Box<dyn Error>> {
        let receiver = self.mdns_daemon.browse(SERVICE_NAME)?;

        loop {
            let message = self.communication_receiver.try_recv();

            if let Ok(message) = message {
                match message {
                    ThreadCommunication::AnswerToLookupRequest => { self.advertise() },
                    ThreadCommunication::StopAnsweringToLookupRequest => { self.stop_advertising() },
                    ThreadCommunication::Shutdown => {
                        self.stop_advertising();
                        self.mdns_daemon.shutdown().ok();

                        return Ok(())
                    },
                    _ => {}
                }
            }

            let result = receiver.recv();

            if let Ok(result) = result {
                match result {
                    ServiceEvent::ServiceResolved(info) => {
                        let properties = info.get_properties();

                        let id = properties.get("deviceId");

                        if let Some(id) = id {
                            if id.ne(&self.my_device.id) {
                                let name = properties.get("deviceName");
                                let port = properties.get("port");
                                let device_type = properties.get("type");

                                if let (Some(name),
                                    Some(port),
                                    Some(device_type)) = (name, port, device_type) {

                                    let port = port.to_string().parse::<u16>();

                                    if let Ok(port) = port {
                                        let device = DeviceInfo {
                                            id: id.to_string(),
                                            name: name.to_string(),
                                            port,
                                            device_type: device_type.to_string(),
                                            ip_address: "".to_string()
                                        };

                                        let mut is_new_device = false;

                                        if let Ok(mut discovered_devices) = self.discovered_devices.write() {
                                            match discovered_devices.insert(device.id.clone(), device.clone()) {
                                                Some(_) => is_new_device = false,
                                                None => is_new_device = true
                                            };
                                        }

                                        if is_new_device {
                                            if let Some(delegate) = &self.delegate {
                                                if let Ok(delegate) = delegate.lock() {
                                                    delegate.device_added(device);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },

                    ServiceEvent::ServiceRemoved(service_type, fullname) => {
                        if service_type == SERVICE_NAME {
                            let device_id = fullname.split(".").next();

                            let mut was_already_deleted = false;

                            if let Some(device_id) = device_id {
                                if let Ok(mut discovered_devices) = self.discovered_devices.write() {
                                    match discovered_devices.remove(device_id) {
                                        Some(_) => was_already_deleted = false,
                                        None => was_already_deleted = true
                                    }
                                }

                                if !was_already_deleted {
                                    if let Some(delegate) = &self.delegate {
                                        if let Ok(delegate) = delegate.lock() {
                                            delegate.device_removed(device_id.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}
