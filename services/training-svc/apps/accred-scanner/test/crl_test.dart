import 'package:flutter_test/flutter_test.dart';

import 'package:eel_accred_scanner/features/crl/crl_model.dart';

void main() {
  testWidgets('CRL stale alert fires at 31 min [AC-M42-09]', (tester) async {
    final staleCrl = CrlModel(
      generatedAt: DateTime.now().subtract(const Duration(minutes: 31)),
      revokedSerials: const [],
    );

    expect(
      staleCrl.isStale(),
      isTrue,
      reason: 'CRL generated 31 minutes ago must be stale',
    );
  });

  test('isRevoked returns true for serial in revokedSerials', () {
    final crl = CrlModel(
      generatedAt: DateTime.now(),
      revokedSerials: const ['ABC-001', 'XYZ-999'],
    );

    expect(crl.revokedSerials.contains('ABC-001'), isTrue);
    expect(crl.revokedSerials.contains('NOT-HERE'), isFalse);
  });

  test('CRL with 29 min age is not stale', () {
    final freshCrl = CrlModel(
      generatedAt: DateTime.now().subtract(const Duration(minutes: 29)),
      revokedSerials: const [],
    );

    expect(freshCrl.isStale(), isFalse);
  });
}
