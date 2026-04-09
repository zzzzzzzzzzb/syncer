use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceId(pub String);

#[derive(Debug, Clone)]
pub struct PeerDevice {
    pub id: DeviceId,
    pub display_name: String,
}

#[derive(Debug, Clone)]
pub struct ClipboardPayload {
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventVersion(pub u64);

#[derive(Debug, Clone)]
pub struct ClipboardEvent {
    pub event_id: String,
    pub source_device: DeviceId,
    pub version: EventVersion,
    pub payload: ClipboardPayload,
    pub timestamp_ms: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    Idle,
    Running,
}

/// 同步会话状态机。
/// 负责信任设备管理、事件版本推进以及远端事件去重判定。
pub struct SyncSession {
    local_device: DeviceId,
    status: SessionStatus,
    trusted_devices: HashMap<DeviceId, PeerDevice>,
    seen_events: HashSet<String>,
    latest_version_by_device: HashMap<DeviceId, EventVersion>,
    next_local_version: u64,
}

impl SyncSession {
    /// 创建会话，初始状态为 Idle。
    pub fn new(local_device: DeviceId) -> Self {
        log::info!("创建同步会话: device={}", local_device.0);
        Self {
            local_device,
            status: SessionStatus::Idle,
            trusted_devices: HashMap::new(),
            seen_events: HashSet::new(),
            latest_version_by_device: HashMap::new(),
            next_local_version: 1,
        }
    }

    /// 启动会话，允许进入同步流程。
    pub fn start(&mut self) {
        self.status = SessionStatus::Running;
        log::info!("同步会话已启动");
    }

    pub fn status(&self) -> SessionStatus {
        self.status
    }

    /// 注册受信设备，通常在配对成功后调用。
    pub fn register_trusted_device(&mut self, device: PeerDevice) {
        log::info!("注册受信设备: {} ({})", device.display_name, device.id.0);
        self.trusted_devices.insert(device.id.clone(), device);
    }

    pub fn revoke_trusted_device(&mut self, device_id: &DeviceId) {
        let removed = self.trusted_devices.remove(device_id);
        if removed.is_some() {
            log::warn!("已撤销设备信任: {}", device_id.0);
        }
    }

    pub fn trusted_devices(&self) -> Vec<PeerDevice> {
        self.trusted_devices.values().cloned().collect()
    }

    /// 生成本地剪切板事件并推进本地版本号。
    pub fn next_local_event(&mut self, payload: ClipboardPayload) -> ClipboardEvent {
        let version = EventVersion(self.next_local_version);
        self.next_local_version += 1;
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or_default();
        // event_id 在 MVP 中由 device_id + version 组成，便于幂等去重。
        let event_id = format!("{}-{}", self.local_device.0, version.0);
        let event = ClipboardEvent {
            event_id: event_id.clone(),
            source_device: self.local_device.clone(),
            version,
            payload,
            timestamp_ms,
        };
        self.seen_events.insert(event_id);
        self.latest_version_by_device
            .insert(self.local_device.clone(), version);
        log::debug!("生成本地剪切板事件: version={}", version.0);
        event
    }

    /// 判定远端事件是否应被应用：
    /// - 已见过事件直接丢弃
    /// - 版本回退事件直接丢弃
    pub fn should_apply_remote_event(&mut self, event: &ClipboardEvent) -> bool {
        if self.seen_events.contains(&event.event_id) {
            log::debug!("忽略重复事件: {}", event.event_id);
            return false;
        }
        let current = self
            .latest_version_by_device
            .get(&event.source_device)
            .copied()
            .unwrap_or(EventVersion(0));
        if event.version <= current {
            log::warn!(
                "忽略过期事件: device={}, incoming={}, current={}",
                event.source_device.0,
                event.version.0,
                current.0
            );
            return false;
        }
        self.seen_events.insert(event.event_id.clone());
        self.latest_version_by_device
            .insert(event.source_device.clone(), event.version);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_duplicate_event() {
        let mut session = SyncSession::new(DeviceId("local-a".into()));
        let event = session.next_local_event(ClipboardPayload {
            content: "hello".into(),
        });
        assert!(!session.should_apply_remote_event(&event));
    }

    #[test]
    fn accepts_newer_remote_event() {
        let mut session = SyncSession::new(DeviceId("local-a".into()));
        let remote = ClipboardEvent {
            event_id: "remote-1".into(),
            source_device: DeviceId("remote".into()),
            version: EventVersion(1),
            payload: ClipboardPayload {
                content: "new".into(),
            },
            timestamp_ms: 0,
        };
        assert!(session.should_apply_remote_event(&remote));
    }
}
