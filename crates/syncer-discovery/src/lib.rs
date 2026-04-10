use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use mdns_sd::{ServiceDaemon, ServiceEvent};
use syncer_core::{DeviceId, PeerDevice};

pub trait DiscoveryProvider {
    fn start(&mut self);
    fn stop(&mut self);
    fn upsert_device(&mut self, device: PeerDevice);
    fn discovered_devices(&self) -> Vec<PeerDevice>;
}

#[derive(Default)]
pub struct MdnsDiscovery {
    running: bool,
    peers: Arc<Mutex<HashMap<DeviceId, PeerDevice>>>,
    daemon: Option<ServiceDaemon>,
    worker: Option<JoinHandle<()>>,
}

impl MdnsDiscovery {
    fn service_type() -> &'static str {
        "_syncer._udp.local."
    }
}

impl DiscoveryProvider for MdnsDiscovery {
    fn start(&mut self) {
        if self.running {
            return;
        }
        let Ok(daemon) = ServiceDaemon::new() else {
            log::error!("mDNS 守护进程初始化失败");
            return;
        };
        let Ok(receiver) = daemon.browse(Self::service_type()) else {
            log::error!("mDNS 浏览启动失败");
            let _ = daemon.shutdown();
            return;
        };
        let peers = Arc::clone(&self.peers);
        let worker = std::thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        let id = DeviceId(info.get_fullname().to_string());
                        let display_name = info.get_hostname().to_string();
                        if let Ok(mut map) = peers.lock() {
                            map.insert(id.clone(), PeerDevice { id, display_name });
                        }
                    }
                    ServiceEvent::ServiceRemoved(_, fullname) => {
                        if let Ok(mut map) = peers.lock() {
                            map.remove(&DeviceId(fullname));
                        }
                    }
                    _ => {}
                }
            }
        });
        self.running = true;
        self.daemon = Some(daemon);
        self.worker = Some(worker);
        log::info!("mDNS 发现服务已启动");
    }

    fn stop(&mut self) {
        if !self.running {
            return;
        }
        self.running = false;
        if let Some(daemon) = self.daemon.take() {
            let _ = daemon.shutdown();
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        log::info!("mDNS 发现服务已停止");
    }

    fn upsert_device(&mut self, device: PeerDevice) {
        if self.running {
            log::debug!("发现设备: {} ({})", device.display_name, device.id.0);
            if let Ok(mut peers) = self.peers.lock() {
                peers.insert(device.id.clone(), device);
            }
        }
    }

    fn discovered_devices(&self) -> Vec<PeerDevice> {
        self.peers
            .lock()
            .map(|peers| peers.values().cloned().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_peers_when_running() {
        let mut discovery = MdnsDiscovery::default();
        discovery.start();
        discovery.upsert_device(PeerDevice {
            id: DeviceId("dev-1".into()),
            display_name: "Laptop".into(),
        });
        assert_eq!(discovery.discovered_devices().len(), 1);
        discovery.stop();
    }
}
