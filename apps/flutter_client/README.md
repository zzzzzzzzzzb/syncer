# flutter_client

Syncer Flutter 客户端 MVP 工程。

## 页面覆盖
- P01 启动与权限引导页
- P02 首页（同步状态总览）
- P03 设备发现页
- P04 配对流程页
- P05 设备详情页
- P06 同步记录页
- P07 冲突处理页
- P08 设置页
- P09 安全与信任管理页
- P10 异常状态页

## 架构概览
- `lib/src/state/app_state.dart`：应用状态、设备与同步记录模型。
- `lib/src/ui/syncer_root.dart`：桌面侧边栏与移动底部 Tab 的自适应壳层。
- `lib/src/ui/pages/`：按页面拆分的 UI 实现。
- `lib/src/ffi/syncer_native.dart`：`dart:ffi` 动态库绑定与 `SyncerFacade` 生命周期管理。

## 当前接入状态
- 已在应用启动时初始化 Rust `syncer-ffi` 动态库并调用 `start_service`。
- 已接入 `status` 拉取并映射到首页同步状态。
- 若本地动态库不可用，客户端自动退回本地 UI 演示态并保持可运行。

## 后续接入
- 将设备列表、同步记录改为 Rust 事件流驱动。
- 把配对、重连、撤销信任等按钮动作改为调用 Rust Core 接口。
