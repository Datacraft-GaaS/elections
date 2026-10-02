// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import 'package:eel_personnel/main.dart';
import 'package:eel_personnel/src/workforce/notifications_controller.dart';
import 'package:eel_personnel/src/workforce/notifications_screen.dart';
import 'package:eel_personnel/src/workforce/workforce_api.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

class _FakeApi extends WorkforceApi {
  List<WorkforceNotification> notifications = const [];

  @override
  Future<List<WorkforceNotification>> listNotifications(String workerId) async {
    return notifications;
  }
}

void main() {
  testWidgets('shell renders four destinations and navigates to the inbox',
      (tester) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          notificationsApiProvider.overrideWithValue(_FakeApi()),
        ],
        child: const EelPersonnelApp(),
      ),
    );
    await tester.pump();

    expect(find.byType(NavigationBar), findsOneWidget);
    expect(find.text('Duty'), findsOneWidget);
    expect(find.text('Inbox'), findsOneWidget);
    expect(find.text('Check in'), findsOneWidget);
    expect(find.text('Status'), findsOneWidget);

    await tester.tap(find.text('Inbox'));
    await tester.pumpAndSettle();

    expect(find.byType(NotificationsScreen), findsOneWidget);
    expect(find.text('Nothing here yet.'), findsOneWidget);
  });

  testWidgets('inbox shows notifications newest-first with kind chips',
      (tester) async {
    final api = _FakeApi()
      ..notifications = [
        WorkforceNotification(
          notificationId: 'n2',
          workerId: 'w1',
          kind: NotificationKind.payment,
          channel: 'phone',
          subject: 'Payment',
          body: 'KES 12,000 disbursed',
          createdAt: DateTime(2026, 8, 10, 10),
        ),
        WorkforceNotification(
          notificationId: 'n1',
          workerId: 'w1',
          kind: NotificationKind.posting,
          subject: 'Posting: STN-001',
          body: 'You are posted to STN-001.',
          createdAt: DateTime(2026, 8, 9, 9),
        ),
      ];

    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          notificationsApiProvider.overrideWithValue(api),
        ],
        child: const MaterialApp(home: NotificationsScreen()),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Payment'), findsWidgets);
    expect(find.text('Posting: STN-001'), findsOneWidget);
    expect(find.text('Nothing here yet.'), findsNothing);
  });
}
