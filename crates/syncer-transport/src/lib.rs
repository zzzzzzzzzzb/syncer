use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

use syncer_core::{ClipboardEvent, ClipboardPayload, DeviceId, EventVersion};

#[derive(Debug, Clone)]
pub enum TransportMessage {
    Hello,
    PairInit { pairing_code: String },
    PairConfirm,
    ClipboardUpdate(ClipboardEvent),
    Ack { event_id: String },
    Heartbeat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportErrorCode {
    InvalidPairingCode,
    BindFailed,
    ConnectFailed,
    NotPaired,
    HandshakeTimeout,
    HandshakeFailed,
    SendFailed,
    RecvFailed,
    DecodeFailed,
    InvalidMessageField,
}

#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    pub handshake_retries: u8,
    pub send_retries: u8,
    pub retry_delay_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            handshake_retries: 20,
            send_retries: 3,
            retry_delay_ms: 5,
        }
    }
}

pub trait SecureChannel {
    fn pair_with_code(&mut self, pairing_code: &str) -> Result<(), TransportErrorCode>;
    fn send(&mut self, message: TransportMessage) -> Result<(), TransportErrorCode>;
    fn recv(&mut self) -> Result<Option<TransportMessage>, TransportErrorCode>;
    fn is_paired(&self) -> bool;
}

pub struct UdpSecureChannel {
    paired: bool,
    socket: UdpSocket,
    session_key: Option<Vec<u8>>,
    retry_policy: RetryPolicy,
}

impl Default for UdpSecureChannel {
    fn default() -> Self {
        let socket = UdpSocket::bind("127.0.0.1:0").expect("failed to bind udp socket");
        let local_addr = socket.local_addr().expect("failed to read local address");
        socket.connect(local_addr).expect("failed to connect udp socket");
        socket
            .set_read_timeout(Some(Duration::from_millis(5)))
            .expect("failed to set read timeout");
        Self {
            paired: false,
            socket,
            session_key: None,
            retry_policy: RetryPolicy::default(),
        }
    }
}

impl SecureChannel for UdpSecureChannel {
    fn pair_with_code(&mut self, pairing_code: &str) -> Result<(), TransportErrorCode> {
        if !is_valid_pairing_code(pairing_code) {
            log::warn!("配对失败，配对码格式无效");
            return Err(TransportErrorCode::InvalidPairingCode);
        }
        self.paired = false;
        self.session_key = None;
        let key = derive_session_key(pairing_code);
        let init = encode_plain_message(&TransportMessage::PairInit {
            pairing_code: pairing_code.to_string(),
        })?;
        self.send_raw_with_retry(&init, self.retry_policy.handshake_retries)?;
        let mut init_ok = false;
        for _ in 0..=self.retry_policy.handshake_retries {
            match read_plain_from_socket(&self.socket) {
                Ok(Some(TransportMessage::PairInit { pairing_code: code })) if code == pairing_code => {
                    init_ok = true;
                    break;
                }
                Ok(Some(_)) => {}
                Ok(None) => {}
                Err(_) => {}
            }
            thread::sleep(Duration::from_millis(self.retry_policy.retry_delay_ms));
        }
        if !init_ok {
            log::warn!("配对失败，握手初始化超时");
            return Err(TransportErrorCode::HandshakeTimeout);
        }
        self.session_key = Some(key);
        self.paired = true;
        let confirm = encrypt_payload(
            encode_plain_message(&TransportMessage::PairConfirm)?,
            self.session_key
                .as_ref()
                .ok_or(TransportErrorCode::HandshakeFailed)?,
        );
        self.send_raw_with_retry(&confirm, self.retry_policy.handshake_retries)?;
        for _ in 0..=self.retry_policy.handshake_retries {
            match self.recv()? {
                Some(TransportMessage::PairConfirm) => {
                    log::info!("配对成功，建立加密会话");
                    return Ok(());
                }
                Some(_) => {}
                None => {}
            }
            thread::sleep(Duration::from_millis(self.retry_policy.retry_delay_ms));
        }
        self.paired = false;
        self.session_key = None;
        log::warn!("配对失败，握手确认超时");
        Err(TransportErrorCode::HandshakeTimeout)
    }

    fn send(&mut self, message: TransportMessage) -> Result<(), TransportErrorCode> {
        if !self.paired {
            log::warn!("未配对通道拒绝发送消息");
            return Err(TransportErrorCode::NotPaired);
        }
        validate_message_fields(&message)?;
        let encoded = encode_plain_message(&message)?;
        let encrypted = encrypt_payload(
            encoded,
            self.session_key
                .as_ref()
                .ok_or(TransportErrorCode::HandshakeFailed)?,
        );
        self.send_raw_with_retry(&encrypted, self.retry_policy.send_retries)
    }

    fn recv(&mut self) -> Result<Option<TransportMessage>, TransportErrorCode> {
        if !self.paired {
            return Err(TransportErrorCode::NotPaired);
        }
        let mut buf = [0u8; 8192];
        let size = match self.socket.recv(&mut buf) {
            Ok(size) => size,
            Err(err) if err.kind() == ErrorKind::WouldBlock || err.kind() == ErrorKind::TimedOut => {
                return Ok(None);
            }
            Err(_) => return Err(TransportErrorCode::RecvFailed),
        };
        let encrypted = &buf[..size];
        let decrypted = decrypt_payload(
            encrypted,
            self.session_key
                .as_ref()
                .ok_or(TransportErrorCode::HandshakeFailed)?,
        );
        decode_plain_message(&decrypted).map(Some)
    }

    fn is_paired(&self) -> bool {
        self.paired
    }
}

impl UdpSecureChannel {
    pub fn with_endpoints(
        local_bind_addr: &str,
        peer_addr: &str,
    ) -> Result<Self, TransportErrorCode> {
        let socket = UdpSocket::bind(local_bind_addr).map_err(|_| TransportErrorCode::BindFailed)?;
        socket
            .connect(peer_addr)
            .map_err(|_| TransportErrorCode::ConnectFailed)?;
        socket
            .set_read_timeout(Some(Duration::from_millis(5)))
            .map_err(|_| TransportErrorCode::RecvFailed)?;
        Ok(Self {
            paired: false,
            socket,
            session_key: None,
            retry_policy: RetryPolicy::default(),
        })
    }

    pub fn set_retry_policy(&mut self, retry_policy: RetryPolicy) {
        self.retry_policy = retry_policy;
    }

    fn send_raw_with_retry(
        &self,
        bytes: &[u8],
        retries: u8,
    ) -> Result<(), TransportErrorCode> {
        for attempt in 0..=retries {
            if self.socket.send(bytes).is_ok() {
                return Ok(());
            }
            if attempt < retries {
                thread::sleep(Duration::from_millis(self.retry_policy.retry_delay_ms));
            }
        }
        log::warn!("发送消息失败");
        Err(TransportErrorCode::SendFailed)
    }
}

const MAX_EVENT_ID_LEN: usize = 128;
const MAX_DEVICE_ID_LEN: usize = 64;
const MAX_CLIPBOARD_CONTENT_LEN: usize = 8192;

fn is_valid_pairing_code(pairing_code: &str) -> bool {
    pairing_code.len() == 6 && pairing_code.chars().all(|c| c.is_ascii_digit())
}

fn validate_message_fields(message: &TransportMessage) -> Result<(), TransportErrorCode> {
    match message {
        TransportMessage::PairInit { pairing_code } => {
            if !is_valid_pairing_code(pairing_code) {
                return Err(TransportErrorCode::InvalidPairingCode);
            }
        }
        TransportMessage::ClipboardUpdate(event) => {
            if event.event_id.is_empty() || event.event_id.len() > MAX_EVENT_ID_LEN {
                return Err(TransportErrorCode::InvalidMessageField);
            }
            if event.source_device.0.is_empty() || event.source_device.0.len() > MAX_DEVICE_ID_LEN {
                return Err(TransportErrorCode::InvalidMessageField);
            }
            if event.payload.content.len() > MAX_CLIPBOARD_CONTENT_LEN {
                return Err(TransportErrorCode::InvalidMessageField);
            }
        }
        TransportMessage::Ack { event_id } => {
            if event_id.is_empty() || event_id.len() > MAX_EVENT_ID_LEN {
                return Err(TransportErrorCode::InvalidMessageField);
            }
        }
        _ => {}
    }
    Ok(())
}

fn read_plain_from_socket(socket: &UdpSocket) -> Result<Option<TransportMessage>, TransportErrorCode> {
    let mut buf = [0u8; 8192];
    let size = match socket.recv(&mut buf) {
        Ok(size) => size,
        Err(err) if err.kind() == ErrorKind::WouldBlock || err.kind() == ErrorKind::TimedOut => {
            return Ok(None);
        }
        Err(_) => return Err(TransportErrorCode::RecvFailed),
    };
    decode_plain_message(&buf[..size]).map(Some)
}

fn derive_session_key(pairing_code: &str) -> Vec<u8> {
    let mut key = Vec::with_capacity(32);
    for round in 0..4u64 {
        let mut hasher = DefaultHasher::new();
        pairing_code.hash(&mut hasher);
        round.hash(&mut hasher);
        key.extend_from_slice(&hasher.finish().to_le_bytes());
    }
    key
}

fn encrypt_payload(data: Vec<u8>, key: &[u8]) -> Vec<u8> {
    data.into_iter()
        .enumerate()
        .map(|(idx, byte)| byte ^ key[idx % key.len()])
        .collect()
}

fn decrypt_payload(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(idx, byte)| byte ^ key[idx % key.len()])
        .collect()
}

fn encode_plain_message(message: &TransportMessage) -> Result<Vec<u8>, TransportErrorCode> {
    validate_message_fields(message)?;
    let mut out = Vec::new();
    match message {
        TransportMessage::Hello => out.push(0),
        TransportMessage::PairInit { pairing_code } => {
            out.push(1);
            write_string(&mut out, pairing_code)?;
        }
        TransportMessage::PairConfirm => out.push(2),
        TransportMessage::ClipboardUpdate(event) => {
            out.push(3);
            write_string(&mut out, &event.event_id)?;
            write_string(&mut out, &event.source_device.0)?;
            out.extend_from_slice(&event.version.0.to_le_bytes());
            write_string(&mut out, &event.payload.content)?;
            out.extend_from_slice(&event.timestamp_ms.to_le_bytes());
        }
        TransportMessage::Ack { event_id } => {
            out.push(4);
            write_string(&mut out, event_id)?;
        }
        TransportMessage::Heartbeat => out.push(5),
    }
    Ok(out)
}

fn decode_plain_message(bytes: &[u8]) -> Result<TransportMessage, TransportErrorCode> {
    let (&tag, mut rest) = bytes
        .split_first()
        .ok_or(TransportErrorCode::DecodeFailed)?;
    let message = match tag {
        0 => TransportMessage::Hello,
        1 => {
            let pairing_code = read_string(&mut rest)?;
            TransportMessage::PairInit { pairing_code }
        }
        2 => TransportMessage::PairConfirm,
        3 => {
            let event_id = read_string(&mut rest)?;
            let source_device = DeviceId(read_string(&mut rest)?);
            let version = read_u64(&mut rest)?;
            let payload = ClipboardPayload {
                content: read_string(&mut rest)?,
            };
            let timestamp_ms = read_u128(&mut rest)?;
            TransportMessage::ClipboardUpdate(ClipboardEvent {
                event_id,
                source_device,
                version: EventVersion(version),
                payload,
                timestamp_ms,
            })
        }
        4 => {
            let event_id = read_string(&mut rest)?;
            TransportMessage::Ack { event_id }
        }
        5 => TransportMessage::Heartbeat,
        _ => return Err(TransportErrorCode::DecodeFailed),
    };
    validate_message_fields(&message)?;
    Ok(message)
}

fn write_string(out: &mut Vec<u8>, value: &str) -> Result<(), TransportErrorCode> {
    if value.len() > u32::MAX as usize {
        return Err(TransportErrorCode::InvalidMessageField);
    }
    let bytes = value.as_bytes();
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(bytes);
    Ok(())
}

fn read_string(input: &mut &[u8]) -> Result<String, TransportErrorCode> {
    let len = read_u32(input)? as usize;
    if input.len() < len {
        return Err(TransportErrorCode::DecodeFailed);
    }
    let (value, rest) = input.split_at(len);
    *input = rest;
    String::from_utf8(value.to_vec()).map_err(|_| TransportErrorCode::DecodeFailed)
}

fn read_u32(input: &mut &[u8]) -> Result<u32, TransportErrorCode> {
    if input.len() < 4 {
        return Err(TransportErrorCode::DecodeFailed);
    }
    let (value, rest) = input.split_at(4);
    *input = rest;
    Ok(u32::from_le_bytes(
        value
            .try_into()
            .map_err(|_| TransportErrorCode::DecodeFailed)?,
    ))
}

fn read_u64(input: &mut &[u8]) -> Result<u64, TransportErrorCode> {
    if input.len() < 8 {
        return Err(TransportErrorCode::DecodeFailed);
    }
    let (value, rest) = input.split_at(8);
    *input = rest;
    Ok(u64::from_le_bytes(
        value
            .try_into()
            .map_err(|_| TransportErrorCode::DecodeFailed)?,
    ))
}

fn read_u128(input: &mut &[u8]) -> Result<u128, TransportErrorCode> {
    if input.len() < 16 {
        return Err(TransportErrorCode::DecodeFailed);
    }
    let (value, rest) = input.split_at(16);
    *input = rest;
    Ok(u128::from_le_bytes(
        value
            .try_into()
            .map_err(|_| TransportErrorCode::DecodeFailed)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use syncer_core::{ClipboardPayload, DeviceId, EventVersion};
    use std::net::UdpSocket;

    #[test]
    fn pairs_and_transfers_message() {
        let mut channel = UdpSecureChannel::default();
        assert!(channel.pair_with_code("123456").is_ok());
        assert!(channel.send(TransportMessage::Hello).is_ok());
        assert!(matches!(channel.recv(), Ok(Some(TransportMessage::Hello))));
    }

    #[test]
    fn transfers_clipboard_event() {
        let mut channel = UdpSecureChannel::default();
        assert!(channel.pair_with_code("654321").is_ok());
        assert!(channel
            .send(TransportMessage::ClipboardUpdate(ClipboardEvent {
                event_id: "evt-1".into(),
                source_device: DeviceId("local-a".into()),
                version: EventVersion(3),
                payload: ClipboardPayload {
                    content: "hello".into(),
                },
                timestamp_ms: 42,
            }))
            .is_ok());
        let Ok(Some(TransportMessage::ClipboardUpdate(event))) = channel.recv() else {
            panic!("expect clipboard update");
        };
        assert_eq!(event.event_id, "evt-1");
        assert_eq!(event.payload.content, "hello");
        assert_eq!(event.version.0, 3);
    }

    #[test]
    fn rejects_invalid_pairing_code() {
        let mut channel = UdpSecureChannel::default();
        assert_eq!(
            channel.pair_with_code("abc"),
            Err(TransportErrorCode::InvalidPairingCode)
        );
    }

    #[test]
    fn rejects_invalid_fields() {
        let mut channel = UdpSecureChannel::default();
        assert!(channel.pair_with_code("111111").is_ok());
        let oversized_event_id = "x".repeat(129);
        let result = channel.send(TransportMessage::ClipboardUpdate(ClipboardEvent {
            event_id: oversized_event_id,
            source_device: DeviceId("local-a".into()),
            version: EventVersion(1),
            payload: ClipboardPayload {
                content: "ok".into(),
            },
            timestamp_ms: 0,
        }));
        assert_eq!(result, Err(TransportErrorCode::InvalidMessageField));
    }

    #[test]
    fn recv_before_pair_returns_error() {
        let mut channel = UdpSecureChannel::default();
        assert!(matches!(
            channel.recv(),
            Err(TransportErrorCode::NotPaired)
        ));
    }

    #[test]
    fn pairs_between_two_endpoints_and_transfers() {
        let left_bind = UdpSocket::bind("127.0.0.1:0").expect("bind left");
        let right_bind = UdpSocket::bind("127.0.0.1:0").expect("bind right");
        let left_addr = left_bind.local_addr().expect("left addr");
        let right_addr = right_bind.local_addr().expect("right addr");
        drop(left_bind);
        drop(right_bind);

        let mut left = UdpSecureChannel::with_endpoints(
            &left_addr.to_string(),
            &right_addr.to_string(),
        )
        .expect("left channel");
        let mut right = UdpSecureChannel::with_endpoints(
            &right_addr.to_string(),
            &left_addr.to_string(),
        )
        .expect("right channel");

        let left_handle = std::thread::spawn(move || {
            left.pair_with_code("123456").expect("left pair");
            left.send(TransportMessage::Hello).expect("left send");
            let message = left.recv().expect("left recv");
            matches!(message, Some(TransportMessage::PairConfirm) | Some(TransportMessage::Ack { .. }) | Some(TransportMessage::Hello) | None)
        });
        let right_handle = std::thread::spawn(move || {
            right.pair_with_code("123456").expect("right pair");
            let message = right.recv().expect("right recv");
            matches!(message, Some(TransportMessage::Hello))
        });

        assert!(left_handle.join().expect("left join"));
        assert!(right_handle.join().expect("right join"));
    }

    #[test]
    fn decode_failed_for_corrupted_payload() {
        let left_bind = UdpSocket::bind("127.0.0.1:0").expect("bind left");
        let right_bind = UdpSocket::bind("127.0.0.1:0").expect("bind right");
        let left_addr = left_bind.local_addr().expect("left addr");
        let right_addr = right_bind.local_addr().expect("right addr");
        drop(left_bind);
        drop(right_bind);

        let mut left = UdpSecureChannel::with_endpoints(
            &left_addr.to_string(),
            &right_addr.to_string(),
        )
        .expect("left channel");
        let mut right = UdpSecureChannel::with_endpoints(
            &right_addr.to_string(),
            &left_addr.to_string(),
        )
        .expect("right channel");
        let right_handle = std::thread::spawn(move || {
            right.pair_with_code("123456").expect("right pair");
            right
        });
        left.pair_with_code("123456").expect("left pair");
        let mut right = right_handle.join().expect("right join");

        left.socket.send(&[0xFF, 0xAA, 0x01]).expect("inject packet");
        assert!(matches!(
            right.recv(),
            Err(TransportErrorCode::DecodeFailed)
        ));
    }
}
