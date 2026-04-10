import 'package:flutter/material.dart';

import '../../state/app_state.dart';

class SyncHistoryPage extends StatelessWidget {
  const SyncHistoryPage({super.key});

  @override
  Widget build(BuildContext context) {
    final state = SyncerAppScope.of(context);
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Text('同步记录', style: Theme.of(context).textTheme.headlineSmall),
        const SizedBox(height: 10),
        Card(
          child: Column(
            children: [
              for (final record in state.records)
                ListTile(
                  leading: Icon(
                    record.result == SyncStatus.success
                        ? Icons.check_circle
                        : Icons.error,
                    color: record.result == SyncStatus.success
                        ? Colors.green
                        : Colors.red,
                  ),
                  title: Text('${record.time.month}/${record.time.day} ${_formatTime(record.time)}'),
                  subtitle: Text('${record.deviceName} · ${record.length} 字符'),
                  trailing: Text(_statusText(record.result)),
                ),
            ],
          ),
        ),
      ],
    );
  }
}

String _statusText(SyncStatus status) {
  switch (status) {
    case SyncStatus.idle:
      return '空闲';
    case SyncStatus.syncing:
      return '同步中';
    case SyncStatus.success:
      return '已完成';
    case SyncStatus.failed:
      return '失败';
    case SyncStatus.conflict:
      return '冲突';
  }
}

String _formatTime(DateTime time) {
  final hour = time.hour.toString().padLeft(2, '0');
  final minute = time.minute.toString().padLeft(2, '0');
  return '$hour:$minute';
}
