# 项目任务跟踪文档

## 文档用途
- 记录项目所有已完成与待完成任务。
- 每次实现完成后立即更新本文件。
- 作为当前执行状态的唯一跟踪入口。

## 更新规则
- 新任务：写入“待完成任务”并标记优先级。
- 实现完成：从“待完成任务”移动到“已完成任务”。
- 每次更新时追加一条“变更记录”，包含日期与变更摘要。

## 已完成任务
- [x] 初始化 Rust workspace 与五个核心 crate。
- [x] 建立核心模块骨架：core/discovery/transport/clipboard/ffi。
- [x] 接入基础日志（log crate）并在关键路径输出日志。
- [x] 在关键结构与核心方法增加注释（会话状态机、FFI 门面）。
- [x] 建立基础文档目录与说明（根 README、architecture README、flutter_client README）。
- [x] 完成编译与测试验证：`cargo check --workspace`、`cargo test --workspace` 全通过。
- [x] 清理 `crates/*/.gitignore`，统一仅保留项目根目录 `.gitignore`。
- [x] 重建并重新初始化项目根目录 Git 仓库（`.git`）。
- [x] 将 `syncer-discovery` 从 `MdnsDiscoveryStub` 替换为真实 mDNS 发现实现。
- [x] 将 `syncer-transport` 从内存通道替换为真实加密传输实现（对接端到端加密与配对握手）。
- [x] 完成协议细化：错误码、重试语义、消息字段约束。
- [x] 增加信任设备持久化（TrustStore）与撤销流程落地。
- [x] 完成 Flutter 客户端工程初始化并接入 `syncer-ffi`。
- [x] 打通首个端到端联调闭环（双设备：发现→配对→文本同步→Ack）。
- [x] 补齐跨平台测试矩阵与网络异常测试用例。

## 待完成任务
- [ ] 无（当前阶段后端任务已完成，等待下一阶段需求）。

## 变更记录
- 2026-04-10：补充 FFI 查询能力，新增受信设备列表/发现设备列表/同步记录/状态快照 JSON 接口，并更新 FFI API 文档。
- 2026-04-10：完成后端收尾：新增双端点端到端联调测试（含 Ack 闭环）、扩展 Rust FFI 动作接口（配对/同步/轮询/撤销/计数/Ack 查询）并补充 FFI 与测试矩阵文档。
- 2026-04-10：完成 Flutter 客户端最小工程接入，新增 dart:ffi 绑定并接入 Rust `syncer-ffi` 动态库启动与状态查询，完成 Rust 全量 check/test 验证。
- 2026-04-10：完成 TrustStore 持久化与撤销流程，新增本地信任设备存储与加载恢复，补充门面撤销能力并通过全量 check/test。
- 2026-04-10：完成协议细化，新增传输错误码与重试策略，落地消息字段约束并补充协议文档与测试，已通过全量 check/test。
- 2026-04-10：完成 `syncer-transport` 真实加密传输替换，基于 UDP 实现配对握手与消息加解密，完成全量 check/test 验证。
- 2026-04-10：完成 `syncer-discovery` 真实 mDNS 发现实现替换，接入 `mdns-sd` 并通过全量 check/test 验证。
- 2026-04-09：创建任务跟踪文档，补录当前已完成项与下一阶段待完成项。
- 2026-04-09：删除 `crates` 下各子 crate 的 `.gitignore`，统一使用根目录忽略规则。
- 2026-04-09：删除损坏的 `.git` 并执行 `git init`，默认分支切换为 `main`。
