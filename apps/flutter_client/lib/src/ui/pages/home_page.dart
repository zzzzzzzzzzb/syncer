import 'package:flutter/material.dart';

import '../../state/app_state.dart';

class HomePage extends StatefulWidget {
  const HomePage({super.key});

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  final _controller = TextEditingController();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final state = SyncerAppScope.of(context);
    final onlineDevices = state.devices.where((d) => d.isOnline).toList();
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Text(
          '同步状态总览',
          style: Theme.of(context).textTheme.headlineSmall,
        ),
        const SizedBox(height: 12),
        Wrap(
          spacing: 12,
          runSpacing: 12,
          children: [
            _MetricCard(title: '同步状态', value: _statusLabel(state.syncStatus)),
            _MetricCard(title: '冲突待处理', value: '0'),
            _MetricCard(
              title: '失败次数',
              value: '${state.records.where((r) => r.result == SyncStatus.failed).length}',
            ),
          ],
        ),
        const SizedBox(height: 16),
        Card(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('FFI 对接状态', style: Theme.of(context).textTheme.titleMedium),
                const SizedBox(height: 8),
                Text(state.ffiConnected ? '已连接 Rust Core' : '未连接 Rust Core'),
                Text('已信任设备: ${state.trustedDeviceCount}'),
                if (state.lastAckEventId != null)
                  Text('最近 ACK: ${state.lastAckEventId}'),
                if (state.lastError != null)
                  Text(
                    '错误: ${state.lastError}',
                    style: TextStyle(color: Theme.of(context).colorScheme.error),
                  ),
                const SizedBox(height: 10),
                TextField(
                  controller: _controller,
                  decoration: const InputDecoration(
                    hintText: '输入要同步的文本',
                    border: OutlineInputBorder(),
                  ),
                ),
                const SizedBox(height: 8),
                Wrap(
                  spacing: 8,
                  children: [
                    FilledButton(
                      onPressed: () {
                        final text = _controller.text.trim();
                        if (text.isEmpty) {
                          return;
                        }
                        final code = state.syncTextOnce(text);
                        final msg = code >= 0 ? '同步发送成功' : '同步失败，错误码 $code';
                        ScaffoldMessenger.of(
                          context,
                        ).showSnackBar(SnackBar(content: Text(msg)));
                      },
                      child: const Text('发送同步'),
                    ),
                    OutlinedButton(
                      onPressed: () {
                        final code = state.refreshFromNative();
                        final msg = code >= 0 ? '已拉取远端消息' : '拉取失败，错误码 $code';
                        ScaffoldMessenger.of(
                          context,
                        ).showSnackBar(SnackBar(content: Text(msg)));
                      },
                      child: const Text('刷新状态'),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
        const SizedBox(height: 12),
        Card(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('已连接设备', style: Theme.of(context).textTheme.titleMedium),
                const SizedBox(height: 10),
                for (final d in onlineDevices)
                  ListTile(
                    dense: true,
                    contentPadding: EdgeInsets.zero,
                    leading: const Icon(Icons.devices),
                    title: Text(d.name),
                    subtitle: Text('最后同步 ${_formatTime(d.lastSync)}'),
                    trailing: const Chip(label: Text('在线')),
                  ),
              ],
            ),
          ),
        ),
        const SizedBox(height: 12),
        Card(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('最近事件', style: Theme.of(context).textTheme.titleMedium),
                const SizedBox(height: 10),
                for (final r in state.records.take(3))
                  ListTile(
                    dense: true,
                    contentPadding: EdgeInsets.zero,
                    leading: Icon(
                      r.result == SyncStatus.success ? Icons.check_circle : Icons.error,
                      color: r.result == SyncStatus.success ? Colors.green : Colors.red,
                    ),
                    title: Text('已同步：${r.length} 字符 -> ${r.deviceName}'),
                    subtitle: Text(_formatTime(r.time)),
                  ),
              ],
            ),
          ),
        ),
      ],
    );
  }
}

class _MetricCard extends StatelessWidget {
  const _MetricCard({required this.title, required this.value});

  final String title;
  final String value;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 220,
      child: Card(
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(title, style: Theme.of(context).textTheme.bodyMedium),
              const SizedBox(height: 8),
              Text(value, style: Theme.of(context).textTheme.titleLarge),
            ],
          ),
        ),
      ),
    );
  }
}

String _statusLabel(SyncStatus status) {
  switch (status) {
    case SyncStatus.idle:
      return '空闲';
    case SyncStatus.syncing:
      return '同步中';
    case SyncStatus.success:
      return '成功';
    case SyncStatus.failed:
      return '失败';
    case SyncStatus.conflict:
      return '冲突待处理';
  }
}

String _formatTime(DateTime time) {
  final hour = time.hour.toString().padLeft(2, '0');
  final minute = time.minute.toString().padLeft(2, '0');
  return '$hour:$minute';
}
