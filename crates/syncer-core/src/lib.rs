use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerDevice {
    pub id: DeviceId,
    pub display_name: String,
}

#[derive(Debug, Clone)]
pub enum TrustStoreError {
    Io(String),
    Parse(String),
}

impl From<std::io::Error> for TrustStoreError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

pub struct FileTrustStore {
    path: PathBuf,
    devices: HashMap<DeviceId, PeerDevice>,
}

impl FileTrustStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            devices: HashMap::new(),
        }
    }

    pub fn load(&mut self) -> Result<(), TrustStoreError> {
        self.devices.clear();
        if !self.path.exists() {
            return Ok(());
        }
        let content = fs::read_to_string(&self.path)?;
        for (line_no, line) in content.lines().enumerate() {
            if line.is_empty() {
                continue;
            }
            let Some((raw_id, raw_name)) = line.split_once('\t') else {
                return Err(TrustStoreError::Parse(format!(
                    "invalid trust store line {}",
                    line_no + 1
                )));
            };
            let id = decode_field(raw_id)?;
            let display_name = decode_field(raw_name)?;
            let device = PeerDevice {
                id: DeviceId(id),
                display_name,
            };
            self.devices.insert(device.id.clone(), device);
        }
        Ok(())
    }

    pub fn upsert(&mut self, device: PeerDevice) -> Result<(), TrustStoreError> {
        self.devices.insert(device.id.clone(), device);
        self.persist()
    }

    pub fn remove(&mut self, device_id: &DeviceId) -> Result<bool, TrustStoreError> {
        let removed = self.devices.remove(device_id).is_some();
        self.persist()?;
        Ok(removed)
    }

    pub fn devices(&self) -> Vec<PeerDevice> {
        self.devices.values().cloned().collect()
    }

    fn persist(&self) -> Result<(), TrustStoreError> {
        if let Some(parent) = Path::new(&self.path).parent() {
            fs::create_dir_all(parent)?;
        }
        let mut lines = Vec::with_capacity(self.devices.len());
        for device in self.devices.values() {
            lines.push(format!(
                "{}\t{}",
                encode_field(&device.id.0),
                encode_field(&device.display_name)
            ));
        }
        fs::write(&self.path, lines.join("\n"))?;
        Ok(())
    }
}

fn encode_field(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '%' => out.push_str("%25"),
            '\t' => out.push_str("%09"),
            '\n' => out.push_str("%0A"),
            '\r' => out.push_str("%0D"),
            _ => out.push(ch),
        }
    }
    out
}

fn decode_field(value: &str) -> Result<String, TrustStoreError> {
    let mut out = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '%' {
            out.push(ch);
            continue;
        }
        let a = chars
            .next()
            .ok_or_else(|| TrustStoreError::Parse("invalid escape".into()))?;
        let b = chars
            .next()
            .ok_or_else(|| TrustStoreError::Parse("invalid escape".into()))?;
        match (a, b) {
            ('2', '5') => out.push('%'),
            ('0', '9') => out.push('\t'),
            ('0', 'A') => out.push('\n'),
            ('0', 'D') => out.push('\r'),
            _ => return Err(TrustStoreError::Parse("unsupported escape".into())),
        }
    }
    Ok(out)
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

    pub fn is_trusted_device(&self, device_id: &DeviceId) -> bool {
        self.trusted_devices.contains_key(device_id)
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
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

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

    #[test]
    fn trust_store_persists_and_recovers_devices() {
        let path = unique_trust_store_path();
        let mut store = FileTrustStore::new(&path);
        store
            .upsert(PeerDevice {
                id: DeviceId("remote-1".into()),
                display_name: "Phone".into(),
            })
            .expect("persist trust device");

        let mut recovered = FileTrustStore::new(&path);
        recovered.load().expect("load trust store");
        let devices = recovered.devices();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].id.0, "remote-1");

        let _ = fs::remove_file(path);
    }

    #[test]
    fn trust_store_remove_device() {
        let path = unique_trust_store_path();
        let mut store = FileTrustStore::new(&path);
        store
            .upsert(PeerDevice {
                id: DeviceId("remote-2".into()),
                display_name: "Tablet".into(),
            })
            .expect("persist trust device");
        assert!(store
            .remove(&DeviceId("remote-2".into()))
            .expect("remove trusted device"));
        assert!(store.devices().is_empty());

        let _ = fs::remove_file(path);
    }

    fn unique_trust_store_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        env::temp_dir().join(format!("syncer-trust-store-{nanos}.db"))
    }
}
