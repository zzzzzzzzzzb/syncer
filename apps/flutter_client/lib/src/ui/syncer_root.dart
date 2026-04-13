import 'package:flutter/material.dart';

import '../state/app_state.dart';
import 'pages/home_page.dart';

class SyncerRoot extends StatelessWidget {
  const SyncerRoot({super.key});

  @override
  Widget build(BuildContext context) {
    final state = SyncerAppScope.of(context);
    return Scaffold(
      appBar: AppBar(title: const Text('Syncer')),
      body: Column(
        children: [
          _TopStatusBar(state: state),
          const Expanded(child: HomePage()),
        ],
      ),
    );
  }
}

class _TopStatusBar extends StatelessWidget {
  const _TopStatusBar({required this.state});

  final SyncerAppState state;

  @override
  Widget build(BuildContext context) {
    return ColoredBox(
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
        child: Wrap(
          spacing: 12,
          runSpacing: 6,
          crossAxisAlignment: WrapCrossAlignment.center,
          children: [
            _StatusChip(
              label: state.hasNetwork ? '自动连接中' : '连接异常',
              color: state.hasNetwork ? Colors.green : Colors.orange,
              icon: state.hasNetwork ? Icons.wifi : Icons.wifi_off,
            ),
            _StatusChip(
              label: state.hasEncryption ? '加密已开启' : '等待配对',
              color: state.hasEncryption ? Colors.blue : Colors.red,
              icon: Icons.lock,
            ),
            _StatusChip(
              label: '自动同步 ${_formatTime(state.lastSync)}',
              color: Colors.green,
              icon: Icons.schedule,
            ),
          ],
        ),
      ),
    );
  }
}

class _StatusChip extends StatelessWidget {
  const _StatusChip({
    required this.label,
    required this.color,
    required this.icon,
  });

  final String label;
  final Color color;
  final IconData icon;

  @override
  Widget build(BuildContext context) {
    return Chip(
      avatar: Icon(icon, size: 16, color: color),
      label: Text(label),
      side: BorderSide(color: color.withValues(alpha: 0.4)),
      backgroundColor: color.withValues(alpha: 0.08),
    );
  }
}

String _formatTime(DateTime time) {
  final hour = time.hour.toString().padLeft(2, '0');
  final minute = time.minute.toString().padLeft(2, '0');
  return '$hour:$minute';
}
