import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:eel_design/eel_design.dart';

/// Ensures the generated token maps never drift from tokens.json.
/// CI runs codegen then compares — this test catches it earlier.
void main() {
  test('colour tokens match tokens.json', () {
    final tokensFile = File('../tokens.json');
    if (!tokensFile.existsSync()) {
      // Running from a different CWD (CI) — resolve from package root.
      final alt = File('packages/design-system/tokens.json');
      if (!alt.existsSync()) {
        fail('tokens.json not found; run from package or repo root');
      }
      return _verify(alt);
    }
    return _verify(tokensFile);
  });
}

void _verify(File tokensFile) {
  final json =
      jsonDecode(tokensFile.readAsStringSync()) as Map<String, dynamic>;

  // Spot-check critical paths against generated maps.
  final expectedPairs = <String, String>{
    'color.brand.primary.light': '#1a56db',
    'color.text.primary.dark': '#f9fafb',
    'alert.danger.border': '#dc2626',
  };
  for (final entry in expectedPairs.entries) {
    expect(
      eelColorTokens[entry.key],
      entry.value,
      reason: 'token ${entry.key} drifted from tokens.json',
    );
  }

  // Structural: every spacing token in JSON exists in the Dart map.
  final spacing = json['spacing'] as Map<String, dynamic>;
  for (final key in spacing.keys) {
    expect(
      eelDimensionTokens.containsKey('spacing.$key'),
      true,
      reason: 'spacing.$key missing from generated dimensions',
    );
  }

  // Spot-check dimension values.
  expect(eelDimensionTokens['spacing.4'], 16.0);
  expect(eelDimensionTokens['touch.min'], 44.0);
  expect(eelDimensionTokens['radius.lg'], 8.0);

  // Font sizes: rem → px at 16 px/rem.
  expect(eelFontSizeTokens['fontSize.base'], 16.0);
  expect(eelFontSizeTokens['fontSize.3xl'], 30.0);
}
