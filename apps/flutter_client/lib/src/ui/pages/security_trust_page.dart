import 'package:flutter/material.dart';

import '../../state/app_state.dart';

class SecurityTrustPage extends StatelessWidget {
  const SecurityTrustPage({super.key});

  @override
  Widget build(BuildContext context) {
    final state = SyncerAppScope.of(context);
    return Scaffold(
      appBar: AppBar(title: const Text('安全与信任管理')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Text('已信任设备', style: Theme.of(context).textTheme.headlineSmall),
          const SizedBox(height: 10),
          for (final d in state.devices)
            Card(
              child: ListTile(
                title: Text(d.name),
                subtitle: Text('首次配对 ${d.trustedAt.month}/${d.trustedAt.day} · 指纹 ${d.fingerprint}'),
                trailing: Wrap(
                  spacing: 8,
                  children: [
                    TextButton(onPressed: () {}, child: const Text('查看指纹')),
                    TextButton(onPressed: () {}, child: const Text('重新配对')),
                    TextButton(
                      onPressed: () {
                        final code = state.revokeDevice(d.id);
                        final message = code >= 0 ? '已撤销 ${d.name}' : '撤销失败，错误码 $code';
                        ScaffoldMessenger.of(
                          context,
                        ).showSnackBar(SnackBar(content: Text(message)));
                      },
                      style: TextButton.styleFrom(foregroundColor: Colors.red),
                      child: const Text('撤销信任'),
                    ),
                  ],
                ),
              ),
            ),
          const SizedBox(height: 8),
          Card(
            color: Colors.red.withValues(alpha: 0.08),
            child: Padding(
              padding: const EdgeInsets.all(12),
              child: Text('撤销后该设备不可接收或推送同步事件，当前已信任 ${state.trustedDeviceCount} 台'),
            ),
          ),
        ],
      ),
    );
  }
}
