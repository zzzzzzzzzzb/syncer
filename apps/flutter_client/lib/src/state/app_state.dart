import 'package:flutter/material.dart';

import '../ffi/syncer_native.dart';

enum TopLevelTab { home, devices, history, settings, pairing }

enum SyncStatus { idle, syncing, success, failed, conflict }

class DeviceInfo {
  const DeviceInfo({
    required this.id,
    required this.name,
    required this.isOnline,
    required this.lastSync,
    required this.trustedAt,
    required this.fingerprint,
  });

  final String id;
  final String name;
  final bool isOnline;
  final DateTime lastSync;
  final DateTime trustedAt;
  final String fingerprint;
}

class SyncRecord {
  const SyncRecord({
    required this.id,
    required this.deviceName,
    required this.time,
    required this.length,
    required this.result,
  });

  final String id;
  final String deviceName;
  final DateTime time;
  final int length;
  final SyncStatus result;
}

class SyncerAppState extends ChangeNotifier {
  SyncerAppState({
    required this.devices,
    required this.records,
    required this.syncStatus,
    required this.hasEncryption,
    required this.hasNetwork,
    required this.lastSync,
    required this.activeTab,
    required this.discoveryDevices,
  });

  factory SyncerAppState.demo() {
    final now = DateTime.now();
    final devices = <DeviceInfo>[
      DeviceInfo(
        id: 'macbook',
        name: 'MacBook-Pro',
        isOnline: true,
        lastSync: now.subtract(const Duration(minutes: 2)),
        trustedAt: now.subtract(const Duration(days: 8)),
        fingerprint: 'AF:92:18:3D',
      ),
      DeviceInfo(
        id: 'pixel8',
        name: 'Pixel-8',
        isOnline: true,
        lastSync: now.subtract(const Duration(minutes: 3)),
        trustedAt: now.subtract(const Duration(days: 2)),
        fingerprint: '3C:11:BE:F0',
      ),
      DeviceInfo(
        id: 'win11',
        name: 'Win11-Office',
        isOnline: false,
        lastSync: now.subtract(const Duration(hours: 3)),
        trustedAt: now.subtract(const Duration(days: 12)),
        fingerprint: 'DA:45:10:67',
      ),
    ];

    return SyncerAppState(
      devices: devices,
      discoveryDevices: devices,
      records: <SyncRecord>[
        SyncRecord(
          id: 'r1',
          deviceName: 'Pixel-8',
          time: now.subtract(const Duration(minutes: 2)),
          length: 142,
          result: SyncStatus.success,
        ),
        SyncRecord(
          id: 'r2',
          deviceName: 'MacBook-Pro',
          time: now.subtract(const Duration(minutes: 11)),
          length: 88,
          result: SyncStatus.success,
        ),
        SyncRecord(
          id: 'r3',
          deviceName: 'Win11-Office',
          time: now.subtract(const Duration(minutes: 27)),
          length: 66,
          result: SyncStatus.failed,
        ),
      ],
      syncStatus: SyncStatus.success,
      hasEncryption: true,
      hasNetwork: true,
      lastSync: now.subtract(const Duration(minutes: 2)),
      activeTab: TopLevelTab.home,
    );
  }

  final List<DeviceInfo> devices;
  final List<SyncRecord> records;
  final List<DeviceInfo> discoveryDevices;
  SyncStatus syncStatus;
  bool hasEncryption;
  bool hasNetwork;
  DateTime lastSync;
  TopLevelTab activeTab;
  bool ffiConnected = false;
  int trustedDeviceCount = 0;
  String? lastAckEventId;
  String? lastError;
  SyncerNativeClient? nativeClient;

  void setTab(TopLevelTab tab) {
    activeTab = tab;
    notifyListeners();
  }

  void setSyncStatus(SyncStatus status) {
    syncStatus = status;
    notifyListeners();
  }

  void setNetwork(bool value) {
    hasNetwork = value;
    notifyListeners();
  }

  void attachNativeClient(SyncerNativeClient client) {
    nativeClient = client;
    ffiConnected = true;
    trustedDeviceCount = client.trustedDeviceCount();
    notifyListeners();
  }

  void setLastError(String? message) {
    lastError = message;
    notifyListeners();
  }

  int startService() {
    final client = nativeClient;
    if (client == null) {
      return -1;
    }
    final startCode = client.startService();
    if (startCode < 0) {
      lastError = '启动服务失败，错误码 $startCode';
      notifyListeners();
      return startCode;
    }
    final statusCode = client.status();
    if (statusCode == 1) {
      syncStatus = SyncStatus.syncing;
      hasNetwork = true;
      lastError = null;
      notifyListeners();
      return 0;
    }
    return -1;
  }

  int pairDevice({
    required String pairingCode,
    required String peerId,
    required String peerName,
  }) {
    final client = nativeClient;
    if (client == null) {
      return -1;
    }
    final code = client.pairDevice(
      pairingCode: pairingCode,
      peerId: peerId,
      peerName: peerName,
    );
    if (code >= 0) {
      hasEncryption = true;
      hasNetwork = true;
      syncStatus = SyncStatus.success;
      trustedDeviceCount = client.trustedDeviceCount();
      lastSync = DateTime.now();
      lastError = null;
      notifyListeners();
      refreshFromNative();
    } else {
      lastError = '配对失败，错误码 $code';
      notifyListeners();
    }
    return code;
  }

  int syncTextOnce(String content) {
    final client = nativeClient;
    if (client == null) {
      return -1;
    }
    final setCode = client.setLocalClipboardContent(content);
    if (setCode < 0) {
      lastError = '设置本地剪切板失败，错误码 $setCode';
      notifyListeners();
      return setCode;
    }
    final syncCode = client.syncLocalClipboardOnce();
    if (syncCode >= 0) {
      syncStatus = SyncStatus.success;
      lastSync = DateTime.now();
      lastError = null;
      records.insert(
        0,
        SyncRecord(
          id: DateTime.now().millisecondsSinceEpoch.toString(),
          deviceName: 'Remote',
          time: lastSync,
          length: content.length,
          result: SyncStatus.success,
        ),
      );
      notifyListeners();
      refreshFromNative();
    } else {
      lastError = '同步失败，错误码 $syncCode';
      notifyListeners();
    }
    return syncCode;
  }

  int pollRemoteOnce() {
    final client = nativeClient;
    if (client == null) {
      return -1;
    }
    final code = client.pollRemoteOnce();
    if (code >= 0) {
      lastAckEventId = client.lastAckEventId();
      if (lastAckEventId != null) {
        lastSync = DateTime.now();
      }
      lastError = null;
      notifyListeners();
      refreshFromNative();
    } else {
      lastError = '拉取远端消息失败，错误码 $code';
      notifyListeners();
    }
    return code;
  }

  int revokeDevice(String deviceId) {
    final client = nativeClient;
    if (client == null) {
      return -1;
    }
    final code = client.revokeDevice(deviceId);
    if (code >= 0) {
      trustedDeviceCount = client.trustedDeviceCount();
      lastError = null;
      notifyListeners();
      refreshFromNative();
    } else {
      lastError = '撤销信任失败，错误码 $code';
      notifyListeners();
    }
    return code;
  }

  int refreshFromNative() {
    final client = nativeClient;
    if (client == null) {
      return -1;
    }
    var changed = false;
    final snapshot = client.snapshotJson();
    if (snapshot != null) {
      final statusText = snapshot['status']?.toString();
      if (statusText == 'running') {
        if (syncStatus == SyncStatus.idle) {
          syncStatus = SyncStatus.syncing;
          changed = true;
        }
      } else if (statusText == 'idle') {
        if (syncStatus != SyncStatus.idle) {
          syncStatus = SyncStatus.idle;
          changed = true;
        }
      }
      final paired = snapshot['paired'] == true;
      if (hasEncryption != paired) {
        hasEncryption = paired;
        changed = true;
      }
      final trustedCount = _toInt(snapshot['trusted_count']);
      if (trustedCount != null && trustedCount != trustedDeviceCount) {
        trustedDeviceCount = trustedCount;
        changed = true;
      }
      final ackFromSnapshot = snapshot['last_ack_event_id']?.toString();
      if (ackFromSnapshot != null && ackFromSnapshot != lastAckEventId) {
        lastAckEventId = ackFromSnapshot;
        changed = true;
      }
      if (!hasNetwork) {
        hasNetwork = true;
        changed = true;
      }
    } else {
      final statusCode = client.status();
      if (statusCode == 1) {
        if (!hasNetwork) {
          hasNetwork = true;
          changed = true;
        }
      } else if (statusCode < 0 && hasNetwork) {
        hasNetwork = false;
        changed = true;
      }
    }

    final trustedList = client.trustedDeviceListJson();
    final nextTrustedDevices = trustedList.map((item) {
      final id = item['id']?.toString() ?? '';
      return DeviceInfo(
        id: id,
        name: item['name']?.toString() ?? id,
        isOnline: true,
        lastSync: lastSync,
        trustedAt: DateTime.fromMillisecondsSinceEpoch(0),
        fingerprint: '--',
      );
    }).toList(growable: false);
    if (_replaceDeviceListIfChanged(devices, nextTrustedDevices)) {
      changed = true;
    }

    final discoveredList = client.discoveredDeviceListJson();
    final nextDiscoveredDevices = discoveredList.map((item) {
      final id = item['id']?.toString() ?? '';
      return DeviceInfo(
        id: id,
        name: item['name']?.toString() ?? id,
        isOnline: true,
        lastSync: lastSync,
        trustedAt: DateTime.fromMillisecondsSinceEpoch(0),
        fingerprint: '--',
      );
    }).toList(growable: false);
    if (_replaceDeviceListIfChanged(discoveryDevices, nextDiscoveredDevices)) {
      changed = true;
    }

    final recordList = client.syncRecordsJson(limit: 30);
    final nextRecords = recordList.map((item) {
      final result = item['result']?.toString() == 'success'
          ? SyncStatus.success
          : SyncStatus.failed;
      return SyncRecord(
        id: item['event_id']?.toString() ?? DateTime.now().toString(),
        deviceName: item['device_id']?.toString() ?? 'unknown',
        time: DateTime.fromMillisecondsSinceEpoch(
          _toInt(item['timestamp_ms']) ?? DateTime.now().millisecondsSinceEpoch,
        ),
        length: _toInt(item['content_len']) ?? 0,
        result: result,
      );
    }).toList(growable: false);
    if (_replaceRecordListIfChanged(nextRecords)) {
      changed = true;
    }

    final pollCode = client.pollRemoteOnce();
    if (pollCode == 1) {
      final ackId = client.lastAckEventId();
      if (ackId != null && ackId != lastAckEventId) {
        lastAckEventId = ackId;
        lastSync = DateTime.now();
        records.insert(
          0,
          SyncRecord(
            id: '${DateTime.now().millisecondsSinceEpoch}-ack',
            deviceName: 'Remote',
            time: lastSync,
            length: 0,
            result: SyncStatus.success,
          ),
        );
        changed = true;
      }
    } else if (pollCode < 0 && pollCode != -104) {
      if (lastError != '拉取远端消息失败，错误码 $pollCode') {
        lastError = '拉取远端消息失败，错误码 $pollCode';
        changed = true;
      }
    }

    if (changed) {
      notifyListeners();
    }
    return 0;
  }

  int? _toInt(dynamic value) {
    if (value is int) {
      return value;
    }
    if (value is num) {
      return value.toInt();
    }
    if (value is String) {
      return int.tryParse(value);
    }
    return null;
  }

  bool _replaceDeviceListIfChanged(List<DeviceInfo> target, List<DeviceInfo> next) {
    if (target.length == next.length) {
      var same = true;
      for (var i = 0; i < target.length; i++) {
        final left = target[i];
        final right = next[i];
        if (left.id != right.id || left.name != right.name || left.isOnline != right.isOnline) {
          same = false;
          break;
        }
      }
      if (same) {
        return false;
      }
    }
    target
      ..clear()
      ..addAll(next);
    return true;
  }

  bool _replaceRecordListIfChanged(List<SyncRecord> next) {
    if (records.length == next.length) {
      var same = true;
      for (var i = 0; i < records.length; i++) {
        final left = records[i];
        final right = next[i];
        if (left.id != right.id ||
            left.deviceName != right.deviceName ||
            left.length != right.length ||
            left.result != right.result ||
            left.time.millisecondsSinceEpoch != right.time.millisecondsSinceEpoch) {
          same = false;
          break;
        }
      }
      if (same) {
        return false;
      }
    }
    records
      ..clear()
      ..addAll(next);
    return true;
  }
}

class SyncerAppScope extends InheritedNotifier<SyncerAppState> {
  const SyncerAppScope({
    super.key,
    required SyncerAppState notifier,
    required Widget child,
  }) : super(notifier: notifier, child: child);

  static SyncerAppState of(BuildContext context) {
    final scope =
        context.dependOnInheritedWidgetOfExactType<SyncerAppScope>();
    if (scope == null || scope.notifier == null) {
      throw StateError('SyncerAppScope not found');
    }
    return scope.notifier!;
  }
}
