use std::collections::HashMap;

use syncer_core::{DeviceId, PeerDevice};

pub trait DiscoveryProvider {
    fn start(&mut self);
    fn stop(&mut self);
    fn upsert_device(&mut self, device: PeerDevice);
    fn discovered_devices(&self) -> Vec<PeerDevice>;
}

#[derive(Default)]
pub struct MdnsDiscoveryStub {
    running: bool,
    peers: HashMap<DeviceId, PeerDevice>,
}

impl DiscoveryProvider for MdnsDiscoveryStub {
    fn start(&mut self) {
        self.running = true;
        log::info!("mDNS 发现服务已启动");
    }

    fn stop(&mut self) {
        self.running = false;
        log::info!("mDNS 发现服务已停止");
    }

    fn upsert_device(&mut self, device: PeerDevice) {
        if self.running {
            log::debug!("发现设备: {} ({})", device.display_name, device.id.0);
            self.peers.insert(device.id.clone(), device);
        }
    }

    fn discovered_devices(&self) -> Vec<PeerDevice> {
        self.peers.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_peers_when_running() {
        let mut discovery = MdnsDiscoveryStub::default();
        discovery.start();
        discovery.upsert_device(PeerDevice {
            id: DeviceId("dev-1".into()),
            display_name: "Laptop".into(),
        });
        assert_eq!(discovery.discovered_devices().len(), 1);
    }
}
