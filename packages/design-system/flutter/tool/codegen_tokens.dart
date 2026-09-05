// Regenerates lib/src/theme/tokens.dart and lib/src/theme/eel_colors.dart
// from packages/design-system/tokens.json.
//
// Run from packages/design-system/flutter:
//   dart run tool/codegen_tokens.dart
//
// The generated files are committed; CI verifies they are up to date.
import 'dart:convert';
import 'dart:io';

const _repoTokensPath = '../tokens.json';
const _outTokens = 'lib/src/theme/tokens.dart';
const _outColors = 'lib/src/theme/eel_colors.dart';

void main() {
  final json = jsonDecode(File(_repoTokensPath).readAsStringSync())
      as Map<String, dynamic>;

  final colorBuf = StringBuffer()
    ..writeln('// GENERATED FILE - do not edit by hand.')
    ..writeln(
        '// Regenerate with: dart run tool/codegen_tokens.dart (from this directory)')
    ..writeln('// Source of truth: packages/design-system/tokens.json')
    ..writeln('// ignore_for_file: type=lint')
    ..writeln()
    ..writeln('const Map<String, String> eelColorTokens = {');
  _collectColors(json, '', colorBuf);
  colorBuf
    ..writeln('};')
    ..writeln();

  final dimBuf = StringBuffer()
    ..writeln()
    ..writeln('const Map<String, double> eelDimensionTokens = {');
  _collectDimensions(json, '', dimBuf);
  dimBuf
    ..writeln('};')
    ..writeln();

  final sizeBuf = StringBuffer()
    ..writeln('const Map<String, double> eelFontSizeTokens = {');
  _collectFontSizes(json, '', sizeBuf);
  sizeBuf.writeln('};');

  final out = colorBuf.toString() + dimBuf.toString() + sizeBuf.toString();
  File(_outTokens).writeAsStringSync(out);
  // eel_colors.dart hand-maintains typed classes; codegen emits the raw maps
  // only. The typed EelColors/EelTheme are generated once and versioned —
  // token drift is caught by test/token_parity_test.dart.
  stdout.writeln('Wrote $_outTokens ($_outColors parity enforced by tests).');
}

void _collectColors(
  Map<String, dynamic> node,
  String prefix,
  StringBuffer buf,
) {
  node.forEach((key, value) {
    if (key.startsWith('\$')) return;
    final path = prefix.isEmpty ? key : '$prefix.$key';
    if (value is Map<String, dynamic>) {
      if (value.containsKey(r'$value') && value[r'$type'] == 'color') {
        buf.writeln("  '$path': '${value[r'$value']}',");
      } else if (value.containsKey('light') || value.containsKey('dark')) {
        for (final mode in ['light', 'dark']) {
          final m = value[mode] as Map<String, dynamic>?;
          if (m != null && m[r'$type'] == 'color') {
            buf.writeln("  '$path.$mode': '${m[r'$value']}',");
          }
        }
      } else {
        _collectColors(value, path, buf);
      }
    }
  });
}

void _collectDimensions(
  Map<String, dynamic> node,
  String prefix,
  StringBuffer buf,
) {
  node.forEach((key, value) {
    if (key.startsWith('\$')) return;
    final path = prefix.isEmpty ? key : '$prefix.$key';
    if (value is Map<String, dynamic>) {
      if (value.containsKey(r'$value') && value[r'$type'] == 'dimension') {
        // Font sizes are emitted separately (converted rem→px at 16 px/rem).
        if (path.startsWith('typography.fontSize')) return;
        final v = value[r'$value'] as String;
        // Skip env() safe-area expressions; px/rem values convert 1:1
        // (rem at 16 px) to logical pixels.
        if (v.contains('env(')) return;
        final px = double.tryParse(v.replaceAll(RegExp(r'(px|rem)$'), '')) ??
            double.tryParse(v);
        if (px != null) buf.writeln("  '$path': $px,");
      } else if (!value.containsKey('light')) {
        _collectDimensions(value, path, buf);
      }
    }
  });
}

void _collectFontSizes(
  Map<String, dynamic> node,
  String prefix,
  StringBuffer buf,
) {
  final typography = node['typography'] as Map<String, dynamic>?;
  if (typography == null) return;
  final sizes = typography['fontSize'] as Map<String, dynamic>?;
  if (sizes == null) return;
  sizes.forEach((key, value) {
    if (value is Map<String, dynamic> && value.containsKey(r'$value')) {
      final rem = double.tryParse(
        (value[r'$value'] as String).replaceAll(RegExp(r'rem'), ''),
      );
      if (rem != null) buf.writeln("  'fontSize.$key': ${rem * 16},");
    }
  });
}
