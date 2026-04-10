import 'package:flutter_test/flutter_test.dart';

import 'package:flutter_client/src/app.dart';

void main() {
  testWidgets('renders syncer home', (tester) async {
    await tester.pumpWidget(const SyncerApp());
    expect(find.text('Syncer'), findsWidgets);
  });
}
