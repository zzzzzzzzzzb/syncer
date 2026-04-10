# syncer

局域网多端实时同步工具，首期聚焦剪切板同步。

## 当前工程结构
- `crates/syncer-core`：领域模型、会话状态机、事件去重。
- `crates/syncer-discovery`：mDNS 发现层抽象与实现。
- `crates/syncer-transport`：UDP 安全传输通道、配对握手与消息编解码。
- `crates/syncer-clipboard`：剪切板适配抽象与内存实现。
- `crates/syncer-ffi`：面向客户端的 Rust 门面接口。
- `apps/flutter_client`：Flutter 客户端占位目录。
- `docs/architecture`：架构与协议文档占位目录。

## MVP 范围
- 局域网 mDNS 自动发现
- 端到端加密 + 配对码
- 剪切板文本实时同步

## 协议消息
- Hello
- PairInit / PairConfirm
- ClipboardUpdate
- Ack
- Heartbeat

## 协议细化文档
- `docs/architecture/protocol-spec.md`

## 任务跟踪
- 统一任务文档：`docs/architecture/task-tracker.md`
- 约定：每次实现完成后，必须同步更新任务跟踪文档中的“已完成任务 / 待完成任务 / 变更记录”。
