import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'ffi/syncer_native.dart';
import 'state/app_state.dart';
import 'ui/syncer_root.dart';

class SyncerApp extends StatefulWidget {
  const SyncerApp({super.key});

  @override
  State<SyncerApp> createState() => _SyncerAppState();
}

class _SyncerAppState extends State<SyncerApp> with WidgetsBindingObserver {
  late final SyncerAppState appState;
  SyncerNativeClient? nativeClient;
  Timer? _pollTimer;
  Timer? _clipboardTimer;
  String? _lastClipboardText;
  bool _syncingClipboard = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    appState = SyncerAppState.demo();
    try {
      final client = SyncerNativeClient.create();
      final started = client.initAndStart(
        localDeviceId: 'flutter-client',
        trustStorePath: '.syncer/flutter-trust-store.db',
      );
      if (started == 0) {
        nativeClient = client;
        appState.attachNativeClient(client);
        appState.startService();
        _pollTimer = Timer.periodic(const Duration(seconds: 2), (_) {
          appState.refreshFromNative();
        });
        _clipboardTimer = Timer.periodic(const Duration(milliseconds: 800), (_) {
          _syncClipboardIfChanged();
        });
        _syncClipboardIfChanged();
      } else {
        appState.setNetwork(false);
        appState.setLastError('启动服务失败，错误码 $started');
      }
    } catch (e) {
      appState.setNetwork(false);
      appState.setLastError(e.toString());
    }
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      appState.refreshFromNative();
      _syncClipboardIfChanged();
    }
  }

  Future<void> _syncClipboardIfChanged() async {
    if (_syncingClipboard || nativeClient == null) {
      return;
    }
    _syncingClipboard = true;
    try {
      final data = await Clipboard.getData(Clipboard.kTextPlain);
      final text = data?.text?.trim();
      if (text == null || text.isEmpty || text == _lastClipboardText) {
        return;
      }
      _lastClipboardText = text;
      appState.syncTextOnce(text);
    } finally {
      _syncingClipboard = false;
    }
  }

  @override
  void dispose() {
    _pollTimer?.cancel();
    _clipboardTimer?.cancel();
    WidgetsBinding.instance.removeObserver(this);
    nativeClient?.dispose();
    appState.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Syncer',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.blue),
        useMaterial3: true,
      ),
      home: SyncerAppScope(
        notifier: appState,
        child: const SyncerRoot(),
      ),
    );
  }
}
