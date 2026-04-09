use std::collections::VecDeque;

use syncer_core::ClipboardEvent;

#[derive(Debug, Clone)]
pub enum TransportMessage {
    Hello,
    PairInit { pairing_code: String },
    PairConfirm,
    ClipboardUpdate(ClipboardEvent),
    Ack { event_id: String },
    Heartbeat,
}

pub trait SecureChannel {
    fn pair_with_code(&mut self, pairing_code: &str) -> bool;
    fn send(&mut self, message: TransportMessage);
    fn recv(&mut self) -> Option<TransportMessage>;
    fn is_paired(&self) -> bool;
}

#[derive(Default)]
pub struct InMemorySecureChannel {
    paired: bool,
    queue: VecDeque<TransportMessage>,
}

impl SecureChannel for InMemorySecureChannel {
    fn pair_with_code(&mut self, pairing_code: &str) -> bool {
        self.paired = pairing_code.len() == 6 && pairing_code.chars().all(|c| c.is_ascii_digit());
        if self.paired {
            log::info!("配对成功，建立加密会话");
        } else {
            log::warn!("配对失败，配对码格式无效");
        }
        self.paired
    }

    fn send(&mut self, message: TransportMessage) {
        if !self.paired {
            log::warn!("未配对通道拒绝发送消息");
            return;
        }
        self.queue.push_back(message);
    }

    fn recv(&mut self) -> Option<TransportMessage> {
        self.queue.pop_front()
    }

    fn is_paired(&self) -> bool {
        self.paired
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairs_and_transfers_message() {
        let mut channel = InMemorySecureChannel::default();
        assert!(channel.pair_with_code("123456"));
        channel.send(TransportMessage::Hello);
        assert!(matches!(channel.recv(), Some(TransportMessage::Hello)));
    }
}
