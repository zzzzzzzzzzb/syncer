# Rust FFI API（MVP）

## 构造与生命周期
- `syncer_facade_new(local_device_id, trust_store_path) -> *mut SyncerFacade`
- `syncer_facade_new_with_network(local_device_id, trust_store_path, local_bind_addr, peer_addr) -> *mut SyncerFacade`
- `syncer_facade_free(facade)`
- `syncer_string_free(ptr)`

## 服务控制
- `syncer_facade_start_service(facade) -> i32`
- `syncer_facade_status(facade) -> i32`
  - `0`: Idle
  - `1`: Running

## 业务动作
- `syncer_facade_pair_device(facade, pairing_code, peer_id, peer_name) -> i32`
- `syncer_facade_set_local_clipboard_content(facade, content) -> i32`
- `syncer_facade_sync_local_clipboard_once(facade) -> i32`
- `syncer_facade_poll_remote_once(facade) -> i32`
- `syncer_facade_revoke_device(facade, device_id) -> i32`
- `syncer_facade_trusted_device_count(facade) -> i32`
- `syncer_facade_last_ack_event_id(facade) -> *mut c_char`

## 查询接口（JSON）
- `syncer_facade_trusted_device_list_json(facade) -> *mut c_char`
- `syncer_facade_discovered_device_list_json(facade) -> *mut c_char`
- `syncer_facade_sync_records_json(facade, limit) -> *mut c_char`
- `syncer_facade_snapshot_json(facade) -> *mut c_char`
- 字符串由 Rust 分配，必须调用 `syncer_string_free(ptr)` 释放

## 返回码约定
- `0` 或正数：成功（语义由函数定义）
- `-1/-2/...`：入参或空指针错误
- `-100 - transport_error_code`：传输层错误
- `-200`：TrustStore 错误

## 传输层错误映射
- `1`: InvalidPairingCode
- `2`: BindFailed
- `3`: ConnectFailed
- `4`: NotPaired
- `5`: HandshakeTimeout
- `6`: HandshakeFailed
- `7`: SendFailed
- `8`: RecvFailed
- `9`: DecodeFailed
- `10`: InvalidMessageField
