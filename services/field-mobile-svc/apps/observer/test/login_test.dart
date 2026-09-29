import 'package:eel_design/eel_design.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:eel_observer/src/features/auth/auth_provider.dart';
import 'package:eel_observer/src/features/auth/login_screen.dart';
import 'package:eel_observer/src/services/attestation_service.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

/// Hardware-backed and unexpired, so the success path can be exercised.
class _Passing implements EelAttestationClient {
  @override
  Future<AttestationVerdict> attest() async => AttestationVerdict(
        deviceId: 'test',
        tier: AttestationTier.tpm,
        expiresAt: DateTime.now().add(const Duration(hours: 1)),
      );
}

void main() {
  Future<ProviderContainer> pumpLogin(
    WidgetTester tester,
    EelAttestationClient client,
  ) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          attestationServiceProvider.overrideWithValue(
            AttestationService(client: client),
          ),
        ],
        child: const MaterialApp(home: ObserverLoginScreen()),
      ),
    );
    await tester.pumpAndSettle();
    return ProviderScope.containerOf(
      tester.element(find.byType(ObserverLoginScreen)),
    );
  }

  Future<void> fillCredentials(WidgetTester tester) async {
    await tester.enterText(find.byType(EelTextField).first, 'op-1');
    await tester.enterText(find.byType(EelTextField).last, 'secret');
  }

  // login() used to sleep 300ms and set authenticated for any credentials at
  // all, so these pin the attestation gate that replaced it.
  testWidgets('signs in when attestation passes', (tester) async {
    final container = await pumpLogin(tester, _Passing());
    await fillCredentials(tester);
    await tester.tap(find.widgetWithText(EelButton, 'Sign In'));
    await tester.pumpAndSettle();

    final state = container.read(authProvider);
    expect(state.status, AuthStatus.authenticated);
    expect(state.observerId, 'op-1');
  });

  testWidgets('the password field is obscured', (tester) async {
    await pumpLogin(tester, _Passing());
    expect(
      tester.widget<EelTextField>(find.byType(EelTextField).last).obscureText,
      isTrue,
    );
  });

  testWidgets('refuses when attestation is software-tier only', (tester) async {
    final container = await pumpLogin(tester, EelAttestationClientStub());
    await fillCredentials(tester);
    await tester.tap(find.widgetWithText(EelButton, 'Sign In'));
    await tester.pumpAndSettle();

    final state = container.read(authProvider);
    expect(state.status, AuthStatus.unauthenticated);
    expect(state.observerId, isNull);
    expect(state.error, contains('software-tier'));
  });

  testWidgets('refuses an empty form before calling the notifier',
      (tester) async {
    final container = await pumpLogin(tester, _Passing());
    await tester.tap(find.widgetWithText(EelButton, 'Sign In'));
    await tester.pumpAndSettle();

    final state = container.read(authProvider);
    expect(state.status, AuthStatus.unauthenticated);
    expect(state.observerId, isNull);
  });
}
