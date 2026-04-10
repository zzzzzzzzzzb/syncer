import 'package:flutter/material.dart';

import '../../state/app_state.dart';
import 'pairing_flow_page.dart';

class DeviceDiscoveryPage extends StatelessWidget {
  const DeviceDiscoveryPage({super.key});

  @override
  Widget build(BuildContext context) {
    final state = SyncerAppScope.of(context);
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Text('设备发现', style: Theme.of(context).textTheme.headlineSmall),
        const SizedBox(height: 8),
        const Text('选择同网段设备开始配对'),
        const SizedBox(height: 12),
        Card(
          child: Column(
            children: [
              for (final d in state.discoveryDevices)
                ListTile(
                  leading: Icon(
                    d.isOnline ? Icons.lan : Icons.portable_wifi_off,
                    color: d.isOnline ? Colors.green : Colors.orange,
                  ),
                  title: Text(d.name),
                  subtitle: Text('在线状态：${d.isOnline ? '在线' : '离线'}'),
                  trailing: FilledButton(
                    onPressed: d.isOnline
                        ? () => Navigator.of(context).push(
                              MaterialPageRoute(
                                builder: (_) => PairingFlowPage(
                                  seedDeviceId: d.id,
                                  seedDeviceName: d.name,
                                ),
                              ),
                            )
                        : null,
                    child: const Text('配对'),
                  ),
                ),
            ],
          ),
        ),
      ],
    );
  }
}
