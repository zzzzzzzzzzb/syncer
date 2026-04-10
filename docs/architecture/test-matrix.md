# 后端测试矩阵（Rust）

## 平台矩阵
- Windows x64：`cargo check --workspace`、`cargo test --workspace`
- macOS x64/arm64：`cargo check --workspace`、`cargo test --workspace`
- Linux x64：`cargo check --workspace`、`cargo test --workspace`

## 端到端链路
- 双实例 UDP 端点联调：发现/配对/文本同步/Ack
- 受信设备持久化恢复与撤销
- FFI 最小链路与扩展动作函数覆盖

## 网络异常用例
- 未配对发送/接收拒绝
- 握手超时与重试
- 非法字段拒绝
- 破损报文解码失败
- 双端点收发稳定性
