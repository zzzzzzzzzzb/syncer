import 'package:flutter/material.dart';

import '../state/app_state.dart';
import 'pages/conflict_resolution_page.dart';
import 'pages/device_detail_page.dart';
import 'pages/device_discovery_page.dart';
import 'pages/exception_state_page.dart';
import 'pages/home_page.dart';
import 'pages/pairing_flow_page.dart';
import 'pages/security_trust_page.dart';
import 'pages/settings_page.dart';
import 'pages/startup_permissions_page.dart';
import 'pages/sync_history_page.dart';

class SyncerRoot extends StatefulWidget {
  const SyncerRoot({super.key});

  @override
  State<SyncerRoot> createState() => _SyncerRootState();
}

class _SyncerRootState extends State<SyncerRoot> {
  int _mobileIndex = 0;

  @override
  Widget build(BuildContext context) {
    final state = SyncerAppScope.of(context);
    return LayoutBuilder(
      builder: (context, constraints) {
        final isDesktop = constraints.maxWidth >= 900;
        final body = isDesktop
            ? _DesktopScaffold(state: state)
            : _MobileScaffold(
                currentIndex: _mobileIndex,
                onIndexChanged: (index) => setState(() => _mobileIndex = index),
              );
        return body;
      },
    );
  }
}

class _DesktopScaffold extends StatelessWidget {
  const _DesktopScaffold({required this.state});

  final SyncerAppState state;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Row(
          children: [
            NavigationRail(
              selectedIndex: state.activeTab.index,
              onDestinationSelected: (index) =>
                  state.setTab(TopLevelTab.values[index]),
              labelType: NavigationRailLabelType.all,
              destinations: const [
                NavigationRailDestination(
                  icon: Icon(Icons.dashboard_outlined),
                  selectedIcon: Icon(Icons.dashboard),
                  label: Text('首页'),
                ),
                NavigationRailDestination(
                  icon: Icon(Icons.devices_other_outlined),
                  selectedIcon: Icon(Icons.devices_other),
                  label: Text('设备'),
                ),
                NavigationRailDestination(
                  icon: Icon(Icons.history_outlined),
                  selectedIcon: Icon(Icons.history),
                  label: Text('记录'),
                ),
                NavigationRailDestination(
                  icon: Icon(Icons.settings_outlined),
                  selectedIcon: Icon(Icons.settings),
                  label: Text('设置'),
                ),
                NavigationRailDestination(
                  icon: Icon(Icons.link_outlined),
                  selectedIcon: Icon(Icons.link),
                  label: Text('配对'),
                ),
              ],
            ),
            const VerticalDivider(width: 1),
            Expanded(
              child: Column(
                children: [
                  _TopStatusBar(state: state),
                  Expanded(child: _desktopPage(state.activeTab)),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _desktopPage(TopLevelTab tab) {
    switch (tab) {
      case TopLevelTab.home:
        return const HomePage();
      case TopLevelTab.devices:
        return const DeviceDiscoveryPage();
      case TopLevelTab.history:
        return const SyncHistoryPage();
      case TopLevelTab.settings:
        return const SettingsPage();
      case TopLevelTab.pairing:
        return const PairingFlowPage();
    }
  }
}

class _MobileScaffold extends StatelessWidget {
  const _MobileScaffold({
    required this.currentIndex,
    required this.onIndexChanged,
  });

  final int currentIndex;
  final ValueChanged<int> onIndexChanged;

  @override
  Widget build(BuildContext context) {
    final state = SyncerAppScope.of(context);
    final pages = const [
      HomePage(),
      DeviceDiscoveryPage(),
      SyncHistoryPage(),
      SettingsPage(),
    ];
    return Scaffold(
      appBar: AppBar(title: const Text('Syncer')),
      body: Column(
        children: [
          _TopStatusBar(state: state),
          Expanded(child: pages[currentIndex]),
        ],
      ),
      bottomNavigationBar: NavigationBar(
        selectedIndex: currentIndex,
        onDestinationSelected: onIndexChanged,
        destinations: const [
          NavigationDestination(icon: Icon(Icons.home_outlined), label: '首页'),
          NavigationDestination(icon: Icon(Icons.devices), label: '设备'),
          NavigationDestination(icon: Icon(Icons.history), label: '记录'),
          NavigationDestination(icon: Icon(Icons.settings), label: '设置'),
        ],
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () => Navigator.of(context).push(
          MaterialPageRoute(builder: (_) => const PairingFlowPage()),
        ),
        label: const Text('开始配对'),
        icon: const Icon(Icons.lock_open),
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
              label: state.hasNetwork ? '网络稳定' : '网络离线',
              color: state.hasNetwork ? Colors.green : Colors.orange,
              icon: state.hasNetwork ? Icons.wifi : Icons.wifi_off,
            ),
            _StatusChip(
              label: state.hasEncryption ? '已加密连接' : '未加密',
              color: state.hasEncryption ? Colors.blue : Colors.red,
              icon: Icons.lock,
            ),
            _StatusChip(
              label: '最近同步 ${_formatTime(state.lastSync)}',
              color: Colors.green,
              icon: Icons.schedule,
            ),
            TextButton.icon(
              onPressed: () => Navigator.of(context).push(
                MaterialPageRoute(builder: (_) => const ExceptionStatePage()),
              ),
              icon: const Icon(Icons.warning_amber_rounded),
              label: const Text('异常状态'),
            ),
            TextButton.icon(
              onPressed: () => Navigator.of(context).push(
                MaterialPageRoute(builder: (_) => const SecurityTrustPage()),
              ),
              icon: const Icon(Icons.verified_user),
              label: const Text('信任管理'),
            ),
            TextButton.icon(
              onPressed: () => Navigator.of(context).push(
                MaterialPageRoute(builder: (_) => const StartupPermissionsPage()),
              ),
              icon: const Icon(Icons.shield_outlined),
              label: const Text('权限引导'),
            ),
            TextButton.icon(
              onPressed: () => Navigator.of(context).push(
                MaterialPageRoute(builder: (_) => const DeviceDetailPage()),
              ),
              icon: const Icon(Icons.info_outline),
              label: const Text('设备详情'),
            ),
            TextButton.icon(
              onPressed: () => Navigator.of(context).push(
                MaterialPageRoute(
                  builder: (_) => const ConflictResolutionPage(),
                ),
              ),
              icon: const Icon(Icons.call_split),
              label: const Text('冲突处理'),
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
