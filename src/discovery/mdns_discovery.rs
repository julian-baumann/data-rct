use std::collections::HashMap;
use std::error::Error;
use crossbeam_channel::{Receiver, Sender};
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use crate::discovery::{DeviceInfo, DiscoveryCommunication, GenericDiscovery, ThreadCommunication};
use crate::PROTOCOL_VERSION;

pub struct MdnsDiscovery {
    mdns_daemon: ServiceDaemon,
    my_device: DeviceInfo,
    discovery_sender: Sender<DiscoveryCommunication>,
    communication_receiver: Receiver<ThreadCommunication>
}

const SERVICE_NAME: &str = "_data-rct._tcp.local.";

impl MdnsDiscovery {
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

impl GenericDiscovery for MdnsDiscovery {
    fn new(my_device: DeviceInfo,
               discovery_sender: Sender<DiscoveryCommunication>,
               communication_receiver: Receiver<ThreadCommunication>) -> Result<MdnsDiscovery, Box<dyn Error>> {
        return Ok(MdnsDiscovery {
            mdns_daemon: ServiceDaemon::new()?,
            my_device,
            discovery_sender,
            communication_receiver
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

                                        self.discovery_sender.try_send(DiscoveryCommunication::DeviceDiscovered(device)).ok();
                                    }
                                }
                            }
                        }

                    }
                    ServiceEvent::ServiceRemoved(service_type, fullname) => {
                        if service_type == SERVICE_NAME {
                            let device_id = fullname.split(".").next();

                            if let Some(device_id) = device_id {
                                self.discovery_sender.try_send(DiscoveryCommunication::RemoveDevice(device_id.to_string())).ok();
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}