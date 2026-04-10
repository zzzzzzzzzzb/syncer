# 传输协议细化（MVP）

## 错误码
- `InvalidPairingCode`：配对码不是 6 位数字。
- `NotPaired`：未建立配对会话即发送或接收业务消息。
- `HandshakeTimeout`：握手阶段在重试窗口内未完成。
- `HandshakeFailed`：握手阶段状态异常或缺少会话密钥。
- `SendFailed`：消息在重试窗口内发送失败。
- `RecvFailed`：消息接收阶段出现非超时异常。
- `DecodeFailed`：消息解码失败（格式非法或字段缺失）。
- `InvalidMessageField`：消息字段违反约束。

## 重试语义
- 握手重试：`handshake_retries = 2`，每次失败后延迟 `5ms` 再重试。
- 发送重试：`send_retries = 2`，每次失败后延迟 `5ms` 再重试。
- 接收超时：读超时返回“无消息”，不作为错误。
- 重试用尽后按阶段返回 `HandshakeTimeout` 或 `SendFailed`。

## 字段约束
- `pairing_code`：必须是 6 位数字。
- `ClipboardUpdate.event_id`：长度 `1..=128`。
- `ClipboardUpdate.source_device`：长度 `1..=64`。
- `ClipboardUpdate.payload.content`：长度 `0..=8192`。
- `Ack.event_id`：长度 `1..=128`。

## 消息类型
- `Hello`
- `PairInit`
- `PairConfirm`
- `ClipboardUpdate`
- `Ack`
- `Heartbeat`
