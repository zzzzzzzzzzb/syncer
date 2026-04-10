import 'dart:convert';
import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

enum SyncerTransportErrorCode {
  invalidPairingCode(1),
  bindFailed(2),
  connectFailed(3),
  notPaired(4),
  handshakeTimeout(5),
  handshakeFailed(6),
  sendFailed(7),
  recvFailed(8),
  decodeFailed(9),
  invalidMessageField(10);

  const SyncerTransportErrorCode(this.code);
  final int code;

  static SyncerTransportErrorCode? fromCode(int code) {
    for (final value in SyncerTransportErrorCode.values) {
      if (value.code == code) {
        return value;
      }
    }
    return null;
  }
}

class SyncerNativeException implements Exception {
  const SyncerNativeException(this.code, this.message);

  final int code;
  final String message;

  @override
  String toString() => 'SyncerNativeException($code): $message';
}

typedef _FacadeNewNative = Pointer<Void> Function(
  Pointer<Utf8> localDeviceId,
  Pointer<Utf8> trustStorePath,
);
typedef _FacadeNewDart = Pointer<Void> Function(
  Pointer<Utf8> localDeviceId,
  Pointer<Utf8> trustStorePath,
);

typedef _FacadeStartNative = Int32 Function(Pointer<Void> facade);
typedef _FacadeStartDart = int Function(Pointer<Void> facade);

typedef _FacadeStatusNative = Int32 Function(Pointer<Void> facade);
typedef _FacadeStatusDart = int Function(Pointer<Void> facade);

typedef _FacadePairDeviceNative = Int32 Function(
  Pointer<Void> facade,
  Pointer<Utf8> pairingCode,
  Pointer<Utf8> peerId,
  Pointer<Utf8> peerName,
);
typedef _FacadePairDeviceDart = int Function(
  Pointer<Void> facade,
  Pointer<Utf8> pairingCode,
  Pointer<Utf8> peerId,
  Pointer<Utf8> peerName,
);

typedef _FacadeSetClipboardNative = Int32 Function(
  Pointer<Void> facade,
  Pointer<Utf8> content,
);
typedef _FacadeSetClipboardDart = int Function(
  Pointer<Void> facade,
  Pointer<Utf8> content,
);

typedef _FacadeSimpleCallNative = Int32 Function(Pointer<Void> facade);
typedef _FacadeSimpleCallDart = int Function(Pointer<Void> facade);

typedef _FacadeRevokeNative = Int32 Function(
  Pointer<Void> facade,
  Pointer<Utf8> deviceId,
);
typedef _FacadeRevokeDart = int Function(
  Pointer<Void> facade,
  Pointer<Utf8> deviceId,
);

typedef _FacadeLastAckNative = Pointer<Utf8> Function(Pointer<Void> facade);
typedef _FacadeLastAckDart = Pointer<Utf8> Function(Pointer<Void> facade);
typedef _FacadeJsonNative = Pointer<Utf8> Function(Pointer<Void> facade);
typedef _FacadeJsonDart = Pointer<Utf8> Function(Pointer<Void> facade);
typedef _FacadeJsonWithLimitNative = Pointer<Utf8> Function(
  Pointer<Void> facade,
  Int32 limit,
);
typedef _FacadeJsonWithLimitDart = Pointer<Utf8> Function(
  Pointer<Void> facade,
  int limit,
);

typedef _StringFreeNative = Void Function(Pointer<Utf8> text);
typedef _StringFreeDart = void Function(Pointer<Utf8> text);

typedef _FacadeFreeNative = Void Function(Pointer<Void> facade);
typedef _FacadeFreeDart = void Function(Pointer<Void> facade);

class SyncerNativeClient {
  SyncerNativeClient._(
    this._newFacade,
    this._startService,
    this._status,
    this._pairDevice,
    this._setLocalClipboardContent,
    this._syncLocalClipboardOnce,
    this._pollRemoteOnce,
    this._revokeDevice,
    this._trustedDeviceCount,
    this._lastAckEventId,
    this._trustedDeviceListJson,
    this._discoveredDeviceListJson,
    this._syncRecordsJson,
    this._snapshotJson,
    this._stringFree,
    this._freeFacade,
  );

  final _FacadeNewDart _newFacade;
  final _FacadeStartDart _startService;
  final _FacadeStatusDart _status;
  final _FacadePairDeviceDart _pairDevice;
  final _FacadeSetClipboardDart _setLocalClipboardContent;
  final _FacadeSimpleCallDart _syncLocalClipboardOnce;
  final _FacadeSimpleCallDart _pollRemoteOnce;
  final _FacadeRevokeDart _revokeDevice;
  final _FacadeSimpleCallDart _trustedDeviceCount;
  final _FacadeLastAckDart _lastAckEventId;
  final _FacadeJsonDart _trustedDeviceListJson;
  final _FacadeJsonDart _discoveredDeviceListJson;
  final _FacadeJsonWithLimitDart _syncRecordsJson;
  final _FacadeJsonDart _snapshotJson;
  final _StringFreeDart _stringFree;
  final _FacadeFreeDart _freeFacade;
  Pointer<Void>? _facade;

  static SyncerNativeClient create() {
    final lib = _openLibrary();
    return SyncerNativeClient._(
      lib.lookupFunction<_FacadeNewNative, _FacadeNewDart>('syncer_facade_new'),
      lib.lookupFunction<_FacadeStartNative, _FacadeStartDart>(
        'syncer_facade_start_service',
      ),
      lib.lookupFunction<_FacadeStatusNative, _FacadeStatusDart>(
        'syncer_facade_status',
      ),
      lib.lookupFunction<_FacadePairDeviceNative, _FacadePairDeviceDart>(
        'syncer_facade_pair_device',
      ),
      lib.lookupFunction<_FacadeSetClipboardNative, _FacadeSetClipboardDart>(
        'syncer_facade_set_local_clipboard_content',
      ),
      lib.lookupFunction<_FacadeSimpleCallNative, _FacadeSimpleCallDart>(
        'syncer_facade_sync_local_clipboard_once',
      ),
      lib.lookupFunction<_FacadeSimpleCallNative, _FacadeSimpleCallDart>(
        'syncer_facade_poll_remote_once',
      ),
      lib.lookupFunction<_FacadeRevokeNative, _FacadeRevokeDart>(
        'syncer_facade_revoke_device',
      ),
      lib.lookupFunction<_FacadeSimpleCallNative, _FacadeSimpleCallDart>(
        'syncer_facade_trusted_device_count',
      ),
      lib.lookupFunction<_FacadeLastAckNative, _FacadeLastAckDart>(
        'syncer_facade_last_ack_event_id',
      ),
      lib.lookupFunction<_FacadeJsonNative, _FacadeJsonDart>(
        'syncer_facade_trusted_device_list_json',
      ),
      lib.lookupFunction<_FacadeJsonNative, _FacadeJsonDart>(
        'syncer_facade_discovered_device_list_json',
      ),
      lib.lookupFunction<_FacadeJsonWithLimitNative, _FacadeJsonWithLimitDart>(
        'syncer_facade_sync_records_json',
      ),
      lib.lookupFunction<_FacadeJsonNative, _FacadeJsonDart>(
        'syncer_facade_snapshot_json',
      ),
      lib.lookupFunction<_StringFreeNative, _StringFreeDart>('syncer_string_free'),
      lib.lookupFunction<_FacadeFreeNative, _FacadeFreeDart>('syncer_facade_free'),
    );
  }

  static DynamicLibrary _openLibrary() {
    final libraryName = _libraryFileName();
    final candidates = _candidateLibraryPaths(libraryName);
    for (final path in candidates) {
      if (File(path).existsSync()) {
        return DynamicLibrary.open(path);
      }
    }
    try {
      return DynamicLibrary.open(libraryName);
    } catch (_) {
      final detail = candidates.map((item) => ' - $item').join('\n');
      throw UnsupportedError('找不到 syncer-ffi 动态库:\n$detail');
    }
  }

  static String _libraryFileName() {
    if (Platform.isWindows) {
      return 'syncer_ffi.dll';
    }
    if (Platform.isMacOS) {
      return 'libsyncer_ffi.dylib';
    }
    if (Platform.isLinux) {
      return 'libsyncer_ffi.so';
    }
    throw UnsupportedError('Unsupported platform for FFI');
  }

  static List<String> _candidateLibraryPaths(String libraryName) {
    final current = Directory.current.path;
    String joinPath(List<String> parts) => parts.join(Platform.pathSeparator);
    return [
      joinPath([current, '..', '..', 'target', 'debug', libraryName]),
      joinPath([current, '..', '..', '..', 'target', 'debug', libraryName]),
      joinPath([current, '..', '..', '..', '..', 'target', 'debug', libraryName]),
      joinPath([current, 'target', 'debug', libraryName]),
      joinPath([current, libraryName]),
    ];
  }

  int initAndStart({required String localDeviceId, required String trustStorePath}) {
    final local = localDeviceId.toNativeUtf8();
    final trust = trustStorePath.toNativeUtf8();
    try {
      _facade = _newFacade(local, trust);
      if (_facade == null || _facade!.address == 0) {
        return -1;
      }
      return _startService(_facade!);
    } finally {
      malloc.free(local);
      malloc.free(trust);
    }
  }

  int status() {
    return _status(_requiredFacade());
  }

  int startService() {
    return _startService(_requiredFacade());
  }

  int pairDevice({
    required String pairingCode,
    required String peerId,
    required String peerName,
  }) {
    final codeText = pairingCode.toNativeUtf8();
    final idText = peerId.toNativeUtf8();
    final nameText = peerName.toNativeUtf8();
    try {
      return _pairDevice(_requiredFacade(), codeText, idText, nameText);
    } finally {
      malloc.free(codeText);
      malloc.free(idText);
      malloc.free(nameText);
    }
  }

  int setLocalClipboardContent(String content) {
    final text = content.toNativeUtf8();
    try {
      return _setLocalClipboardContent(_requiredFacade(), text);
    } finally {
      malloc.free(text);
    }
  }

  int syncLocalClipboardOnce() {
    return _syncLocalClipboardOnce(_requiredFacade());
  }

  int pollRemoteOnce() {
    return _pollRemoteOnce(_requiredFacade());
  }

  int revokeDevice(String deviceId) {
    final idText = deviceId.toNativeUtf8();
    try {
      return _revokeDevice(_requiredFacade(), idText);
    } finally {
      malloc.free(idText);
    }
  }

  int trustedDeviceCount() {
    return _trustedDeviceCount(_requiredFacade());
  }

  String? lastAckEventId() {
    final ptr = _lastAckEventId(_requiredFacade());
    if (ptr.address == 0) {
      return null;
    }
    try {
      return ptr.toDartString();
    } finally {
      _stringFree(ptr);
    }
  }

  List<Map<String, dynamic>> trustedDeviceListJson() {
    return _readJsonList(_trustedDeviceListJson(_requiredFacade()));
  }

  List<Map<String, dynamic>> discoveredDeviceListJson() {
    return _readJsonList(_discoveredDeviceListJson(_requiredFacade()));
  }

  List<Map<String, dynamic>> syncRecordsJson({int limit = 20}) {
    return _readJsonList(_syncRecordsJson(_requiredFacade(), limit));
  }

  Map<String, dynamic>? snapshotJson() {
    return _readJsonMap(_snapshotJson(_requiredFacade()));
  }

  void ensureSuccess(int code, {required String action}) {
    if (code >= 0) {
      return;
    }
    if (code <= -101 && code >= -110) {
      final transportCode = -100 - code;
      final transportError = SyncerTransportErrorCode.fromCode(transportCode);
      final message = transportError == null
          ? '传输层错误($transportCode)'
          : '传输层错误: ${transportError.name}';
      throw SyncerNativeException(code, '$action 失败，$message');
    }
    if (code == -200) {
      throw SyncerNativeException(code, '$action 失败，TrustStore 错误');
    }
    throw SyncerNativeException(code, '$action 失败，返回码 $code');
  }

  Pointer<Void> _requiredFacade() {
    final facade = _facade;
    if (facade == null || facade.address == 0) {
      throw const SyncerNativeException(-1, 'FFI 未初始化');
    }
    return facade;
  }

  List<Map<String, dynamic>> _readJsonList(Pointer<Utf8> ptr) {
    if (ptr.address == 0) {
      return const [];
    }
    try {
      final raw = ptr.toDartString();
      final decoded = jsonDecode(raw);
      if (decoded is List) {
        return decoded.whereType<Map>().map((e) {
          final map = <String, dynamic>{};
          for (final entry in e.entries) {
            final key = entry.key;
            if (key is String) {
              map[key] = entry.value;
            }
          }
          return map;
        }).toList(growable: false);
      }
      return const [];
    } catch (_) {
      return const [];
    } finally {
      _stringFree(ptr);
    }
  }

  Map<String, dynamic>? _readJsonMap(Pointer<Utf8> ptr) {
    if (ptr.address == 0) {
      return null;
    }
    try {
      final raw = ptr.toDartString();
      final decoded = jsonDecode(raw);
      if (decoded is Map) {
        final map = <String, dynamic>{};
        for (final entry in decoded.entries) {
          final key = entry.key;
          if (key is String) {
            map[key] = entry.value;
          }
        }
        return map;
      }
      return null;
    } catch (_) {
      return null;
    } finally {
      _stringFree(ptr);
    }
  }

  void dispose() {
    final facade = _facade;
    if (facade == null || facade.address == 0) {
      return;
    }
    _freeFacade(facade);
    _facade = null;
  }
}
