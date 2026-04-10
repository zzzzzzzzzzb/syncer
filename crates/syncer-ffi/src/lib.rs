use std::path::PathBuf;
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    time::{SystemTime, UNIX_EPOCH},
};

use syncer_clipboard::{ClipboardAdapter, MemoryClipboard};
use syncer_core::{
    ClipboardPayload, DeviceId, FileTrustStore, PeerDevice, SessionStatus, SyncSession,
    TrustStoreError,
};
use syncer_discovery::{DiscoveryProvider, MdnsDiscovery};
use syncer_transport::{SecureChannel, TransportErrorCode, TransportMessage, UdpSecureChannel};

#[derive(Debug)]
pub enum FacadeError {
    Transport(TransportErrorCode),
    TrustStore(TrustStoreError),
}

#[derive(Debug, Clone, Copy)]
enum SyncRecordDirection {
    Outbound,
    Inbound,
}

#[derive(Debug, Clone, Copy)]
enum SyncRecordResult {
    Success,
    Dropped,
}

#[derive(Debug, Clone)]
struct SyncRecord {
    event_id: String,
    device_id: String,
    direction: SyncRecordDirection,
    result: SyncRecordResult,
    content_len: usize,
    timestamp_ms: u128,
}

fn parse_optional_cstr(input: *const c_char) -> Option<String> {
    if input.is_null() {
        return None;
    }
    let value = unsafe { CStr::from_ptr(input) };
    Some(value.to_string_lossy().into_owned())
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default()
}

fn json_escape(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out
}

fn into_c_string_ptr(value: String) -> *mut c_char {
    CString::new(value)
        .map(|text| text.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

fn facade_error_to_code(error: FacadeError) -> i32 {
    match error {
        FacadeError::Transport(code) => -100 - transport_error_to_code(code),
        FacadeError::TrustStore(_) => -200,
    }
}

fn transport_error_to_code(error: TransportErrorCode) -> i32 {
    match error {
        TransportErrorCode::InvalidPairingCode => 1,
        TransportErrorCode::BindFailed => 2,
        TransportErrorCode::ConnectFailed => 3,
        TransportErrorCode::NotPaired => 4,
        TransportErrorCode::HandshakeTimeout => 5,
        TransportErrorCode::HandshakeFailed => 6,
        TransportErrorCode::SendFailed => 7,
        TransportErrorCode::RecvFailed => 8,
        TransportErrorCode::DecodeFailed => 9,
        TransportErrorCode::InvalidMessageField => 10,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn syncer_facade_new(
    local_device_id: *const c_char,
    trust_store_path: *const c_char,
) -> *mut SyncerFacade {
    let local_id = parse_optional_cstr(local_device_id).unwrap_or_else(|| "flutter-client".into());
    let path = parse_optional_cstr(trust_store_path)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".syncer/trust-store.db"));
    Box::into_raw(Box::new(SyncerFacade::new_with_trust_store_path(local_id, path)))
}

#[unsafe(no_mangle)]
pub extern "C" fn syncer_facade_new_with_network(
    local_device_id: *const c_char,
    trust_store_path: *const c_char,
    local_bind_addr: *const c_char,
    peer_addr: *const c_char,
) -> *mut SyncerFacade {
    let local_id = parse_optional_cstr(local_device_id).unwrap_or_else(|| "flutter-client".into());
    let path = parse_optional_cstr(trust_store_path)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".syncer/trust-store.db"));
    let bind = parse_optional_cstr(local_bind_addr).unwrap_or_else(|| "127.0.0.1:0".into());
    let peer = parse_optional_cstr(peer_addr).unwrap_or_else(|| "127.0.0.1:0".into());
    let facade = SyncerFacade::new_with_network(local_id, path, &bind, &peer)
        .unwrap_or_else(|_| SyncerFacade::new("flutter-client"));
    Box::into_raw(Box::new(facade))
}

#[unsafe(no_mangle)]
pub extern "C" fn syncer_facade_start_service(facade: *mut SyncerFacade) -> i32 {
    if facade.is_null() {
        return -1;
    }
    let facade = unsafe { &mut *facade };
    facade.start_service();
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn syncer_facade_status(facade: *const SyncerFacade) -> i32 {
    if facade.is_null() {
        return -1;
    }
    let facade = unsafe { &*facade };
    match facade.status() {
        SessionStatus::Idle => 0,
        SessionStatus::Running => 1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn syncer_facade_pair_device(
    facade: *mut SyncerFacade,
    pairing_code: *const c_char,
    peer_id: *const c_char,
    peer_name: *const c_char,
) -> i32 {
    if facade.is_null() {
        return -1;
    }
    let Some(code) = parse_optional_cstr(pairing_code) else {
        return -2;
    };
    let Some(id) = parse_optional_cstr(peer_id) else {
        return -3;
    };
    let name = parse_optional_cstr(peer_name).unwrap_or_else(|| id.clone());
    let facade = unsafe { &mut *facade };
    match facade.pair_device(
        &code,
        PeerDevice {
            id: DeviceId(id),
            display_name: name,
        },
    ) {
        Ok(()) => 0,
        Err(err) => facade_error_to_code(err),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn syncer_facade_set_local_clipboard_content(
    facade: *mut SyncerFacade,
    content: *const c_char,
) -> i32 {
    if facade.is_null() {
        return -1;
    }
    let Some(content) = parse_optional_cstr(content) else {
        return -2;
    };
    let facade = unsafe { &mut *facade };
    facade.set_local_clipboard_content(content);
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn syncer_facade_sync_local_clipboard_once(facade: *mut SyncerFacade) -> i32 {
    if facade.is_null() {
        return -1;
    }
    let facade = unsafe { &mut *facade };
    match facade.sync_local_clipboard_once() {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(err) => facade_error_to_code(err),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn syncer_facade_poll_remote_once(facade: *mut SyncerFacade) -> i32 {
    if facade.is_null() {
        return -1;
    }
    let facade = unsafe { &mut *facade };
    match facade.poll_remote_once() {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(err) => facade_error_to_code(err),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn syncer_facade_revoke_device(
    facade: *mut SyncerFacade,
    device_id: *const c_char,
) -> i32 {
    if facade.is_null() {
        return -1;
    }
    let Some(device_id) = parse_optional_cstr(device_id) else {
        return -2;
    };
    let facade = unsafe { &mut *facade };
    match facade.revoke_device(&device_id) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(err) => facade_error_to_code(err),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn syncer_facade_trusted_device_count(facade: *const SyncerFacade) -> i32 {
    if facade.is_null() {
        return -1;
    }
    let facade = unsafe { &*facade };
    facade.trusted_devices().len() as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn syncer_facade_last_ack_event_id(facade: *const SyncerFacade) -> *mut c_char {
    if facade.is_null() {
        return std::ptr::null_mut();
    }
    let facade = unsafe { &*facade };
    let Some(event_id) = facade.last_ack_event_id() else {
        return std::ptr::null_mut();
    };
    CString::new(event_id)
        .map(|text| text.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn syncer_facade_trusted_device_list_json(
    facade: *const SyncerFacade,
) -> *mut c_char {
    if facade.is_null() {
        return std::ptr::null_mut();
    }
    let facade = unsafe { &*facade };
    into_c_string_ptr(facade.trusted_devices_json())
}

#[unsafe(no_mangle)]
pub extern "C" fn syncer_facade_discovered_device_list_json(
    facade: *const SyncerFacade,
) -> *mut c_char {
    if facade.is_null() {
        return std::ptr::null_mut();
    }
    let facade = unsafe { &*facade };
    into_c_string_ptr(facade.discovered_devices_json())
}

#[unsafe(no_mangle)]
pub extern "C" fn syncer_facade_sync_records_json(
    facade: *const SyncerFacade,
    limit: i32,
) -> *mut c_char {
    if facade.is_null() {
        return std::ptr::null_mut();
    }
    let facade = unsafe { &*facade };
    let limit = if limit <= 0 { 20 } else { limit as usize };
    into_c_string_ptr(facade.sync_records_json(limit))
}

#[unsafe(no_mangle)]
pub extern "C" fn syncer_facade_snapshot_json(facade: *const SyncerFacade) -> *mut c_char {
    if facade.is_null() {
        return std::ptr::null_mut();
    }
    let facade = unsafe { &*facade };
    into_c_string_ptr(facade.snapshot_json())
}

#[unsafe(no_mangle)]
pub extern "C" fn syncer_string_free(value: *mut c_char) {
    if value.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(value));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn syncer_facade_free(facade: *mut SyncerFacade) {
    if facade.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(facade));
    }
}

impl From<TransportErrorCode> for FacadeError {
    fn from(value: TransportErrorCode) -> Self {
        Self::Transport(value)
    }
}

impl From<TrustStoreError> for FacadeError {
    fn from(value: TrustStoreError) -> Self {
        Self::TrustStore(value)
    }
}

/// 对客户端暴露的高层门面。
/// 当前以同步调用为主，后续可替换为异步事件流接口。
pub struct SyncerFacade {
    session: SyncSession,
    discovery: MdnsDiscovery,
    transport: UdpSecureChannel,
    clipboard: MemoryClipboard,
    trust_store: FileTrustStore,
    last_ack_event_id: Option<String>,
    sync_records: Vec<SyncRecord>,
}

impl SyncerFacade {
    /// 初始化本地设备实例。
    pub fn new(local_device_id: impl Into<String>) -> Self {
        let default_path = PathBuf::from(".syncer/trust-store.db");
        Self::new_with_trust_store_path(local_device_id, default_path)
    }

    pub fn new_with_trust_store_path(
        local_device_id: impl Into<String>,
        trust_store_path: impl Into<PathBuf>,
    ) -> Self {
        let mut trust_store = FileTrustStore::new(trust_store_path);
        let mut session = SyncSession::new(DeviceId(local_device_id.into()));
        if let Err(err) = trust_store.load() {
            log::warn!("加载 TrustStore 失败: {:?}", err);
        }
        for device in trust_store.devices() {
            session.register_trusted_device(device);
        }
        Self {
            session,
            discovery: MdnsDiscovery::default(),
            transport: UdpSecureChannel::default(),
            clipboard: MemoryClipboard::default(),
            trust_store,
            last_ack_event_id: None,
            sync_records: Vec::new(),
        }
    }

    pub fn new_with_network(
        local_device_id: impl Into<String>,
        trust_store_path: impl Into<PathBuf>,
        local_bind_addr: &str,
        peer_addr: &str,
    ) -> Result<Self, TransportErrorCode> {
        let mut trust_store = FileTrustStore::new(trust_store_path);
        let mut session = SyncSession::new(DeviceId(local_device_id.into()));
        if let Err(err) = trust_store.load() {
            log::warn!("加载 TrustStore 失败: {:?}", err);
        }
        for device in trust_store.devices() {
            session.register_trusted_device(device);
        }
        Ok(Self {
            session,
            discovery: MdnsDiscovery::default(),
            transport: UdpSecureChannel::with_endpoints(local_bind_addr, peer_addr)?,
            clipboard: MemoryClipboard::default(),
            trust_store,
            last_ack_event_id: None,
            sync_records: Vec::new(),
        })
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
    pub fn pair_device(
        &mut self,
        pairing_code: &str,
        peer: PeerDevice,
    ) -> Result<(), FacadeError> {
        self.transport.pair_with_code(pairing_code)?;
        self.session.register_trusted_device(peer.clone());
        if let Err(err) = self.trust_store.upsert(peer.clone()) {
            self.session.revoke_trusted_device(&peer.id);
            return Err(err.into());
        }
        self.discovery.upsert_device(peer);
        Ok(())
    }

    pub fn revoke_device(&mut self, device_id: &str) -> Result<bool, FacadeError> {
        let id = DeviceId(device_id.to_string());
        self.session.revoke_trusted_device(&id);
        self.trust_store.remove(&id).map_err(Into::into)
    }

    pub fn trusted_devices(&self) -> Vec<PeerDevice> {
        self.session.trusted_devices()
    }

    pub fn current_clipboard_content(&self) -> String {
        self.clipboard.read_current().content
    }

    pub fn last_ack_event_id(&self) -> Option<String> {
        self.last_ack_event_id.clone()
    }

    pub fn trusted_devices_json(&self) -> String {
        let mut items = Vec::new();
        for device in self.session.trusted_devices() {
            items.push(format!(
                "{{\"id\":\"{}\",\"name\":\"{}\"}}",
                json_escape(&device.id.0),
                json_escape(&device.display_name),
            ));
        }
        format!("[{}]", items.join(","))
    }

    pub fn discovered_devices_json(&self) -> String {
        let mut items = Vec::new();
        for device in self.discovery.discovered_devices() {
            items.push(format!(
                "{{\"id\":\"{}\",\"name\":\"{}\"}}",
                json_escape(&device.id.0),
                json_escape(&device.display_name),
            ));
        }
        format!("[{}]", items.join(","))
    }

    pub fn sync_records_json(&self, limit: usize) -> String {
        let mut items = Vec::new();
        for record in self.sync_records.iter().rev().take(limit) {
            let direction = match record.direction {
                SyncRecordDirection::Outbound => "outbound",
                SyncRecordDirection::Inbound => "inbound",
            };
            let result = match record.result {
                SyncRecordResult::Success => "success",
                SyncRecordResult::Dropped => "dropped",
            };
            items.push(format!(
                "{{\"event_id\":\"{}\",\"device_id\":\"{}\",\"direction\":\"{}\",\"result\":\"{}\",\"content_len\":{},\"timestamp_ms\":{}}}",
                json_escape(&record.event_id),
                json_escape(&record.device_id),
                direction,
                result,
                record.content_len,
                record.timestamp_ms
            ));
        }
        format!("[{}]", items.join(","))
    }

    pub fn snapshot_json(&self) -> String {
        let status = match self.status() {
            SessionStatus::Idle => "idle",
            SessionStatus::Running => "running",
        };
        let paired = self.transport.is_paired();
        let trusted_count = self.session.trusted_devices().len();
        let discovered_count = self.discovery.discovered_devices().len();
        let last_ack = self
            .last_ack_event_id
            .as_ref()
            .map(|v| format!("\"{}\"", json_escape(v)))
            .unwrap_or_else(|| "null".into());
        format!(
            "{{\"status\":\"{}\",\"paired\":{},\"trusted_count\":{},\"discovered_count\":{},\"last_ack_event_id\":{},\"record_count\":{}}}",
            status,
            if paired { "true" } else { "false" },
            trusted_count,
            discovered_count,
            last_ack,
            self.sync_records.len()
        )
    }

    pub fn set_local_clipboard_content(&mut self, content: impl Into<String>) {
        self.clipboard.set_local_content(content.into());
    }

    /// 将本地剪切板内容封装为同步事件并发送。
    pub fn sync_local_clipboard_once(&mut self) -> Result<bool, FacadeError> {
        if !self.transport.is_paired() {
            log::warn!("未配对，忽略本地同步请求");
            return Err(TransportErrorCode::NotPaired.into());
        }
        let payload = self.clipboard.read_current();
        let event = self.session.next_local_event(payload);
        let event_id = event.event_id.clone();
        let device_id = event.source_device.0.clone();
        let content_len = event.payload.content.len();
        self.transport
            .send(TransportMessage::ClipboardUpdate(event))?;
        self.sync_records.push(SyncRecord {
            event_id,
            device_id,
            direction: SyncRecordDirection::Outbound,
            result: SyncRecordResult::Success,
            content_len,
            timestamp_ms: now_ms(),
        });
        Ok(true)
    }

    /// 拉取一次远端消息并尝试应用。
    pub fn poll_remote_once(&mut self) -> Result<bool, FacadeError> {
        let Some(message) = self.transport.recv()? else {
            return Ok(false);
        };
        if let TransportMessage::ClipboardUpdate(event) = message {
            if !self.session.is_trusted_device(&event.source_device) {
                self.sync_records.push(SyncRecord {
                    event_id: event.event_id,
                    device_id: event.source_device.0,
                    direction: SyncRecordDirection::Inbound,
                    result: SyncRecordResult::Dropped,
                    content_len: event.payload.content.len(),
                    timestamp_ms: now_ms(),
                });
                return Ok(false);
            }
            if self.session.should_apply_remote_event(&event) {
                self.clipboard
                    .write_remote_content(ClipboardPayload { content: event.payload.content });
                self.transport.send(TransportMessage::Ack {
                    event_id: event.event_id.clone(),
                })?;
                self.sync_records.push(SyncRecord {
                    event_id: event.event_id,
                    device_id: event.source_device.0,
                    direction: SyncRecordDirection::Inbound,
                    result: SyncRecordResult::Success,
                    content_len: self.clipboard.read_current().content.len(),
                    timestamp_ms: now_ms(),
                });
                return Ok(true);
            }
        } else if let TransportMessage::Ack { event_id } = message {
            self.last_ack_event_id = Some(event_id);
            return Ok(true);
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::net::UdpSocket;
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};
    use syncer_core::DeviceId;

    #[test]
    fn starts_and_pairs() {
        let path = unique_trust_store_path();
        let mut facade = SyncerFacade::new_with_trust_store_path("local", &path);
        facade.start_service();
        assert!(matches!(facade.status(), SessionStatus::Running));
        let paired = facade.pair_device(
            "123456",
            PeerDevice {
                id: DeviceId("remote-1".into()),
                display_name: "Phone".into(),
            },
        );
        assert!(paired.is_ok());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn persists_and_revokes_trusted_device() {
        let path = unique_trust_store_path();
        let mut facade = SyncerFacade::new_with_trust_store_path("local", &path);
        facade
            .pair_device(
                "123456",
                PeerDevice {
                    id: DeviceId("remote-2".into()),
                    display_name: "Tablet".into(),
                },
            )
            .expect("pair and persist");
        drop(facade);

        let mut recovered = SyncerFacade::new_with_trust_store_path("local", &path);
        assert_eq!(recovered.trusted_devices().len(), 1);
        assert!(recovered.revoke_device("remote-2").expect("revoke device"));
        assert!(recovered.trusted_devices().is_empty());

        let reloaded = SyncerFacade::new_with_trust_store_path("local", &path);
        assert!(reloaded.trusted_devices().is_empty());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn two_facades_complete_e2e_sync_with_ack() {
        let left_bind = UdpSocket::bind("127.0.0.1:0").expect("bind left");
        let right_bind = UdpSocket::bind("127.0.0.1:0").expect("bind right");
        let left_addr = left_bind.local_addr().expect("left addr");
        let right_addr = right_bind.local_addr().expect("right addr");
        drop(left_bind);
        drop(right_bind);

        let left_path = unique_trust_store_path();
        let right_path = unique_trust_store_path();
        let mut left = SyncerFacade::new_with_network(
            "left",
            &left_path,
            &left_addr.to_string(),
            &right_addr.to_string(),
        )
        .expect("left facade");
        let mut right = SyncerFacade::new_with_network(
            "right",
            &right_path,
            &right_addr.to_string(),
            &left_addr.to_string(),
        )
        .expect("right facade");

        left.start_service();
        right.start_service();
        let left_pair = thread::spawn(move || {
            let mut left = left;
            left.pair_device(
                "123456",
                PeerDevice {
                    id: DeviceId("right".into()),
                    display_name: "Right".into(),
                },
            )
            .expect("left pair");
            left.set_local_clipboard_content("hello-syncer");
            left.sync_local_clipboard_once().expect("left sync");
            left.poll_remote_once().expect("left ack poll");
            left
        });
        right
            .pair_device(
                "123456",
                PeerDevice {
                    id: DeviceId("left".into()),
                    display_name: "Left".into(),
                },
            )
            .expect("right pair");
        right.poll_remote_once().expect("right receive");

        let left = left_pair.join().expect("join left");
        assert_eq!(right.current_clipboard_content(), "hello-syncer");
        assert!(left.last_ack_event_id().is_some());
        assert!(left.sync_records_json(10).contains("outbound"));
        assert!(right.sync_records_json(10).contains("inbound"));
        assert!(right.snapshot_json().contains("\"status\":\"running\""));

        let _ = fs::remove_file(left_path);
        let _ = fs::remove_file(right_path);
    }

    fn unique_trust_store_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        env::temp_dir().join(format!("syncer-ffi-trust-store-{nanos}.db"))
    }
}
