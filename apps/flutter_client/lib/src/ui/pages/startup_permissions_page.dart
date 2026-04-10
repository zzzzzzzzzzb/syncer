import 'package:flutter/material.dart';

class StartupPermissionsPage extends StatefulWidget {
  const StartupPermissionsPage({super.key});

  @override
  State<StartupPermissionsPage> createState() => _StartupPermissionsPageState();
}

class _StartupPermissionsPageState extends State<StartupPermissionsPage> {
  bool clipboard = false;
  bool network = false;

  @override
  Widget build(BuildContext context) {
    final done = clipboard && network;
    return Scaffold(
      appBar: AppBar(title: const Text('启动与权限引导')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('首次使用', style: Theme.of(context).textTheme.headlineSmall),
            const SizedBox(height: 10),
            CheckboxListTile(
              value: clipboard,
              onChanged: (v) => setState(() => clipboard = v ?? false),
              title: const Text('剪切板权限'),
            ),
            CheckboxListTile(
              value: network,
              onChanged: (v) => setState(() => network = v ?? false),
              title: const Text('本地网络权限'),
            ),
            const SizedBox(height: 16),
            FilledButton(
              onPressed: done ? () => Navigator.of(context).pop() : null,
              child: const Text('进入设备发现'),
            ),
          ],
        ),
      ),
    );
  }
}
