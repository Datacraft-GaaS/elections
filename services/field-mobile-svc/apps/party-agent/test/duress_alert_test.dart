import 'package:eel_design/eel_design.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:eel_party_agent/src/features/auth/auth_provider.dart';
import 'package:eel_party_agent/src/features/duress_alert/duress_alert_screen.dart';
import 'package:eel_party_agent/src/services/attestation_service.dart';
import 'package:eel_party_agent/src/features/duress_alert/duress_alert_service.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

/// Records what the beacon was asked to send, and can be made to fail.
class _RecordingDuressService implements DuressAlertService {
  _RecordingDuressService({this.failWith});

  final Object? failWith;
  final sent = <Map<String, String>>[];

  @override
  Future<void> sendSilentAlert({
    required String agentId,
    required String stationCode,
  }) async {
    if (failWith != null) throw failWith!;
    sent.add({'agentId': agentId, 'stationCode': stationCode});
  }
}

class _Passing implements EelAttestationClient {
  @override
  Future<AttestationVerdict> attest() async => AttestationVerdict(
        deviceId: 'test',
        tier: AttestationTier.tpm,
        expiresAt: DateTime.now().add(const Duration(hours: 1)),
      );
}

void main() {
  Future<ProviderContainer> pump(
    WidgetTester tester, {
    required DuressAlertService service,
    String? agentId = 'agent-001',
  }) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          duressAlertServiceProvider.overrideWithValue(service),
          authProvider.overrideWith((ref) => AuthNotifier(
            AttestationService(client: _Passing()),
          )),
        ],
        child: MaterialApp(home: Scaffold(
          body: Consumer(
            builder: (context, ref, _) => Column(
              children: [
                ElevatedButton(
                  onPressed: () => ref.read(authProvider.notifier).login(
                        agentId ?? '',
                        'pin',
                      ),
                  child: const Text('sign-in'),
                ),
                const Expanded(child: DuressAlertScreen()),
              ],
            ),
          ),
        )),
      ),
    );
    await tester.pumpAndSettle();
    return ProviderScope.containerOf(
      tester.element(find.byType(DuressAlertScreen)),
    );
  }

  Future<void> signIn(WidgetTester tester, [String id = 'agent-001']) async {
    await tester.tap(find.text('sign-in'));
    await tester.pumpAndSettle();
  }

  // The SEND DURESS ALERT button had an empty handler and DuressAlertService
  // had no call site at all, so the beacon could never be sent.
  testWidgets('sends the beacon with the signed-in agent and station',
      (tester) async {
    final service = _RecordingDuressService();
    await pump(tester, service: service);
    await signIn(tester);

    await tester.enterText(find.byType(EelTextField), 'KE-NBI-001');
    await tester.tap(find.widgetWithText(EelButton, 'SEND DURESS ALERT'));
    await tester.pumpAndSettle();

    expect(service.sent, hasLength(1));
    expect(service.sent.single['agentId'], 'agent-001');
    expect(service.sent.single['stationCode'], 'KE-NBI-001');
  });

  testWidgets('reports a failure instead of implying it was sent',
      (tester) async {
    final service = _RecordingDuressService(failWith: 'no mTLS client');
    await pump(tester, service: service);
    await signIn(tester);

    await tester.enterText(find.byType(EelTextField), 'KE-NBI-001');
    await tester.tap(find.widgetWithText(EelButton, 'SEND DURESS ALERT'));
    await tester.pumpAndSettle();

    expect(service.sent, isEmpty);
    final banner = find.byType(EelBanner);
    expect(
      find.descendant(
        of: banner.last,
        matching: find.textContaining('was not delivered'),
      ),
      findsOneWidget,
    );
  });

  testWidgets('refuses without a station code and sends nothing',
      (tester) async {
    final service = _RecordingDuressService();
    await pump(tester, service: service);
    await signIn(tester);

    await tester.tap(find.widgetWithText(EelButton, 'SEND DURESS ALERT'));
    await tester.pumpAndSettle();

    expect(service.sent, isEmpty);
    expect(
      find.descendant(
        of: find.byType(EelBanner).last,
        matching: find.textContaining('Enter the station'),
      ),
      findsOneWidget,
    );
  });

  testWidgets('refuses when nobody is signed in', (tester) async {
    final service = _RecordingDuressService();
    await pump(tester, service: service, agentId: null);

    await tester.enterText(find.byType(EelTextField), 'KE-NBI-001');
    await tester.tap(find.widgetWithText(EelButton, 'SEND DURESS ALERT'));
    await tester.pumpAndSettle();

    expect(service.sent, isEmpty);
  });
}
