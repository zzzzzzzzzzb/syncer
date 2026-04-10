import 'package:flutter/material.dart';

class ConflictResolutionPage extends StatelessWidget {
  const ConflictResolutionPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('冲突处理')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('检测到同步冲突', style: Theme.of(context).textTheme.headlineSmall),
            const SizedBox(height: 8),
            const Text('请选择保留本机文本或接受远端文本'),
            const SizedBox(height: 16),
            Card(
              child: Padding(
                padding: const EdgeInsets.all(12),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: const [
                    Text('本机文本'),
                    SizedBox(height: 4),
                    Text('本地复制内容片段...'),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 8),
            Card(
              child: Padding(
                padding: const EdgeInsets.all(12),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: const [
                    Text('远端文本'),
                    SizedBox(height: 4),
                    Text('远端设备同步内容片段...'),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),
            Wrap(
              spacing: 10,
              children: [
                FilledButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: const Text('保留本机'),
                ),
                OutlinedButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: const Text('接受远端'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
