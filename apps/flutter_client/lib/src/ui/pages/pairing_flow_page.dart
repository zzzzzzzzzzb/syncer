import 'package:flutter/material.dart';

import '../../state/app_state.dart';

class PairingFlowPage extends StatefulWidget {
  const PairingFlowPage({
    super.key,
    this.seedDeviceId,
    this.seedDeviceName,
  });

  final String? seedDeviceId;
  final String? seedDeviceName;

  @override
  State<PairingFlowPage> createState() => _PairingFlowPageState();
}

class _PairingFlowPageState extends State<PairingFlowPage> {
  final _codeController = TextEditingController();
  int _step = 0;
  String? _selectedDeviceId;

  @override
  void initState() {
    super.initState();
    _selectedDeviceId = widget.seedDeviceId;
  }

  @override
  void dispose() {
    _codeController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final state = SyncerAppScope.of(context);
    final availableDevices = state.discoveryDevices;
    return Scaffold(
      appBar: AppBar(title: const Text('配对流程')),
      body: Stepper(
        currentStep: _step,
        onStepContinue: _next,
        onStepCancel: _prev,
        controlsBuilder: (context, details) {
          return Row(
            children: [
              FilledButton(onPressed: details.onStepContinue, child: const Text('下一步')),
              const SizedBox(width: 12),
              TextButton(onPressed: details.onStepCancel, child: const Text('上一步')),
            ],
          );
        },
        steps: [
          Step(
            title: const Text('选择目标设备'),
            content: DropdownButtonFormField<String>(
              initialValue: _selectedDeviceId,
              items: availableDevices
                  .map((d) => DropdownMenuItem(value: d.id, child: Text(d.name)))
                  .toList(),
              onChanged: (v) => setState(() => _selectedDeviceId = v),
            ),
            isActive: _step >= 0,
          ),
          Step(
            title: const Text('输入 6 位配对码'),
            content: TextField(
              controller: _codeController,
              keyboardType: TextInputType.number,
              maxLength: 6,
              decoration: const InputDecoration(
                hintText: '请输入 6 位数字',
                border: OutlineInputBorder(),
              ),
            ),
            isActive: _step >= 1,
          ),
          Step(
            title: const Text('确认设备指纹'),
            content: Card(
              child: ListTile(
                title: Text(_selectedDeviceName(state) ?? '未选择设备'),
                subtitle: const Text('本机: 8B:20:CA:F1  对端: AF:92:18:3D'),
              ),
            ),
            isActive: _step >= 2,
          ),
          Step(
            title: const Text('配对结果'),
            content: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text('已建立加密连接，可以开始同步'),
                const SizedBox(height: 8),
                Wrap(
                  spacing: 8,
                  children: [
                    FilledButton(
                      onPressed: () => Navigator.of(context).pop(),
                      child: const Text('去首页'),
                    ),
                    OutlinedButton(
                      onPressed: () => Navigator.of(context).pop(),
                      child: const Text('查看设备详情'),
                    ),
                  ],
                ),
              ],
            ),
            isActive: _step >= 3,
          ),
        ],
      ),
    );
  }

  void _next() {
    final state = SyncerAppScope.of(context);
    if (_step == 0 && _selectedDeviceId == null) {
      _showError('请选择目标设备');
      return;
    }
    if (_step == 1 && !_isValidPairCode(_codeController.text)) {
      _showError('请输入正确的 6 位数字配对码');
      return;
    }
    if (_step == 2) {
      final peerId = _selectedDeviceId!;
      final peerName = _selectedDeviceName(state) ?? widget.seedDeviceName ?? peerId;
      final code = state.pairDevice(
        pairingCode: _codeController.text,
        peerId: peerId,
        peerName: peerName,
      );
      if (code < 0) {
        _showError('配对失败，错误码 $code');
        return;
      }
    }
    if (_step < 3) {
      setState(() => _step += 1);
    }
  }

  void _prev() {
    if (_step > 0) {
      setState(() => _step -= 1);
    }
  }

  bool _isValidPairCode(String input) {
    final reg = RegExp(r'^\d{6}$');
    return reg.hasMatch(input);
  }

  String? _selectedDeviceName(SyncerAppState state) {
    for (final device in state.discoveryDevices) {
      if (device.id == _selectedDeviceId) {
        return device.name;
      }
    }
    return widget.seedDeviceName;
  }

  void _showError(String msg) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(msg)),
    );
  }
}
