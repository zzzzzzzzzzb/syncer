import 'package:flutter/material.dart';

class SettingsPage extends StatefulWidget {
  const SettingsPage({super.key});

  @override
  State<SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends State<SettingsPage> {
  bool _syncEnabled = true;
  bool _autoStart = true;
  bool _notifyEnabled = true;

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Text('设置', style: Theme.of(context).textTheme.headlineSmall),
        const SizedBox(height: 10),
        Card(
          child: Column(
            children: [
              SwitchListTile(
                value: _syncEnabled,
                onChanged: (v) => setState(() => _syncEnabled = v),
                title: const Text('同步开关'),
              ),
              SwitchListTile(
                value: _autoStart,
                onChanged: (v) => setState(() => _autoStart = v),
                title: const Text('自动启动'),
              ),
              SwitchListTile(
                value: _notifyEnabled,
                onChanged: (v) => setState(() => _notifyEnabled = v),
                title: const Text('通知提醒'),
              ),
            ],
          ),
        ),
      ],
    );
  }
}
