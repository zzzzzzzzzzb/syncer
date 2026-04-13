import 'package:flutter/material.dart';

import '../../state/app_state.dart';

class HomePage extends StatelessWidget {
  const HomePage({super.key});

  @override
  Widget build(BuildContext context) {
    final state = SyncerAppScope.of(context);
    return Center(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 560),
        child: Padding(
          padding: const EdgeInsets.all(20),
          child: Card(
            child: Padding(
              padding: const EdgeInsets.all(20),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('已自动连接并持续同步', style: Theme.of(context).textTheme.titleLarge),
                  const SizedBox(height: 12),
                  _StatusRow(
                    icon: state.hasNetwork ? Icons.wifi : Icons.wifi_off,
                    label: state.hasNetwork ? '连接正常' : '连接中断',
                  ),
                  const SizedBox(height: 8),
                  _StatusRow(
                    icon: Icons.devices,
                    label: '已信任设备 ${state.trustedDeviceCount}',
                  ),
                  const SizedBox(height: 8),
                  _StatusRow(
                    icon: Icons.schedule,
                    label: '最近同步 ${_formatTime(state.lastSync)}',
                  ),
                  if (state.lastError != null) ...[
                    const SizedBox(height: 10),
                    Text(
                      state.lastError!,
                      style: TextStyle(color: Theme.of(context).colorScheme.error),
                    ),
                  ],
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class _StatusRow extends StatelessWidget {
  const _StatusRow({required this.icon, required this.label});

  final IconData icon;
  final String label;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Icon(icon, size: 18),
        const SizedBox(width: 8),
        Text(label),
      ],
    );
  }
}

String _formatTime(DateTime time) {
  final hour = time.hour.toString().padLeft(2, '0');
  final minute = time.minute.toString().padLeft(2, '0');
  return '$hour:$minute';
}
