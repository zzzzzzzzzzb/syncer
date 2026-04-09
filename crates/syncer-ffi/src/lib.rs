use syncer_clipboard::{ClipboardAdapter, MemoryClipboard};
use syncer_core::{ClipboardPayload, DeviceId, PeerDevice, SessionStatus, SyncSession};
use syncer_discovery::{DiscoveryProvider, MdnsDiscoveryStub};
use syncer_transport::{InMemorySecureChannel, SecureChannel, TransportMessage};

/// 对客户端暴露的高层门面。
/// 当前以同步调用为主，后续可替换为异步事件流接口。
pub struct SyncerFacade {
    session: SyncSession,
    discovery: MdnsDiscoveryStub,
    transport: InMemorySecureChannel,
    clipboard: MemoryClipboard,
}

impl SyncerFacade {
    /// 初始化本地设备实例。
    pub fn new(local_device_id: impl Into<String>) -> Self {
        Self {
            session: SyncSession::new(DeviceId(local_device_id.into())),
            discovery: MdnsDiscoveryStub::default(),
            transport: InMemorySecureChannel::default(),
            clipboard: MemoryClipboard::default(),
        }
    }

    /// 启动会话与发现服务。
    pub fn start_service(&mut self) {
        self.session.start();
        self.discovery.start();
        log::info!("Syncer 服务已启动");
    }

    pub fn status(&self) -> SessionStatus {
        self.session.status()
    }

    /// 通过配对码建立信任关系。
    pub fn pair_device(&mut self, pairing_code: &str, peer: PeerDevice) -> bool {
        if !self.transport.pair_with_code(pairing_code) {
            return false;
        }
        self.session.register_trusted_device(peer.clone());
        self.discovery.upsert_device(peer);
        true
    }

    pub fn set_local_clipboard_content(&mut self, content: impl Into<String>) {
        self.clipboard.set_local_content(content.into());
    }

    /// 将本地剪切板内容封装为同步事件并发送。
    pub fn sync_local_clipboard_once(&mut self) -> bool {
        if !self.transport.is_paired() {
            log::warn!("未配对，忽略本地同步请求");
            return false;
        }
        let payload = self.clipboard.read_current();
        let event = self.session.next_local_event(payload);
        self.transport.send(TransportMessage::ClipboardUpdate(event));
        true
    }

    /// 拉取一次远端消息并尝试应用。
    pub fn poll_remote_once(&mut self) -> bool {
        let Some(message) = self.transport.recv() else {
            return false;
        };
        if let TransportMessage::ClipboardUpdate(event) = message {
            if self.session.should_apply_remote_event(&event) {
                self.clipboard
                    .write_remote_content(ClipboardPayload { content: event.payload.content });
                self.transport.send(TransportMessage::Ack {
                    event_id: event.event_id,
                });
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syncer_core::DeviceId;

    #[test]
    fn starts_and_pairs() {
        let mut facade = SyncerFacade::new("local");
        facade.start_service();
        assert!(matches!(facade.status(), SessionStatus::Running));
        let paired = facade.pair_device(
            "123456",
            PeerDevice {
                id: DeviceId("remote-1".into()),
                display_name: "Phone".into(),
            },
        );
        assert!(paired);
    }
}
