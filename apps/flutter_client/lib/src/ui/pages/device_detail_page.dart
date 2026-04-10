import 'package:flutter/material.dart';

import '../../state/app_state.dart';

class DeviceDetailPage extends StatelessWidget {
  const DeviceDetailPage({super.key});

  @override
  Widget build(BuildContext context) {
    final state = SyncerAppScope.of(context);
    final device = state.devices.first;
    return Scaffold(
      appBar: AppBar(title: const Text('设备详情')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          ListTile(
            leading: const Icon(Icons.devices),
            title: Text(device.name),
            subtitle: Text('会话状态：${device.isOnline ? '在线' : '离线'}'),
          ),
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('信任状态', style: Theme.of(context).textTheme.titleMedium),
                  const SizedBox(height: 8),
                  const Text('已信任'),
                  const SizedBox(height: 8),
                  Text('最近同步 ${_formatTime(device.lastSync)}'),
                  const SizedBox(height: 8),
                  Text('指纹 ${device.fingerprint}'),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}

String _formatTime(DateTime time) {
  final hour = time.hour.toString().padLeft(2, '0');
  final minute = time.minute.toString().padLeft(2, '0');
  return '$hour:$minute';
}
