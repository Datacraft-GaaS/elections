import 'package:eel_personnel/src/workforce/workforce_api.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('parses a payment notification from workforce-svc JSON', () {
    final n = WorkforceNotification.fromJson({
      'notification_id': 'n1',
      'worker_id': 'w1',
      'kind': 'payment',
      'channel': 'phone',
      'subject': 'Payment',
      'body': 'KES 12,000 disbursed',
      'delivery_ref': 'a' * 64,
      'created_at': '2026-08-10T10:00:00Z',
    });

    expect(n.notificationId, 'n1');
    expect(n.kind, NotificationKind.payment);
    expect(n.channel, 'phone');
    expect(n.subject, 'Payment');
    expect(n.deliveryRef, 'a' * 64);
    expect(n.createdAt, DateTime.utc(2026, 8, 10, 10));
  });

  test('unknown kind falls back to message', () {
    final n = WorkforceNotification.fromJson({
      'notification_id': 'n2',
      'worker_id': 'w1',
      'kind': 'carrier-pigeon',
      'subject': 'Hi',
      'body': 'Hello',
      'created_at': '2026-08-10T10:00:00Z',
    });

    expect(n.kind, NotificationKind.message);
    expect(n.channel, isNull);
  });

  test('every kind round-trips through its string name', () {
    for (final kind in NotificationKind.values) {
      expect(notificationKindFromString(kind.name), kind);
    }
  });
}
