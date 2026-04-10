import 'package:flutter/material.dart';

import '../../state/app_state.dart';

class ExceptionStatePage extends StatelessWidget {
  const ExceptionStatePage({super.key});

  @override
  Widget build(BuildContext context) {
    final state = SyncerAppScope.of(context);
    return Scaffold(
      appBar: AppBar(title: const Text('异常状态')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('连接中断', style: Theme.of(context).textTheme.headlineSmall),
            const SizedBox(height: 8),
            const Text('心跳超时，连接中断。可尝试一键重连，失败后重新配对。'),
            const SizedBox(height: 16),
            Wrap(
              spacing: 10,
              children: [
                FilledButton.icon(
                  onPressed: () {
                    final code = state.startService();
                    final message = code == 0 ? '已恢复连接' : '重连失败，错误码 $code';
                    _toast(context, message);
                  },
                  icon: const Icon(Icons.refresh),
                  label: const Text('一键重连'),
                ),
                OutlinedButton.icon(
                  onPressed: () => Navigator.of(context).pop(),
                  icon: const Icon(Icons.link_off),
                  label: const Text('重新配对'),
                ),
              ],
            ),
            const SizedBox(height: 18),
            Card(
              child: ListTile(
                leading: const Icon(Icons.error_outline, color: Colors.red),
                title: const Text('握手失败'),
                subtitle: const Text('建议检查配对码是否过期并重新发起配对'),
                trailing: TextButton(
                  onPressed: () {
                    final code = state.pollRemoteOnce();
                    final message = code >= 0 ? '已重试' : '重试失败，错误码 $code';
                    _toast(context, message);
                  },
                  child: const Text('重试'),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  void _toast(BuildContext context, String msg) {
    ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(msg)));
  }
}
