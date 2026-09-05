/// Dart lint rule: detects stub UI widgets that should be replaced with real
/// components. This closes the blind spot where `Text('...stub')` or
/// `Center(child: Text('...stub'))` escapes static analysis because they're
/// not flagged by dart_lints by default.
///
/// Usage: `dart analyze --rules=tools/completeness/rules/dart_stub_ui.dart`
///
/// Example violation:
///   Text('This is a stub screen')  // ❌ triggers rule
///
/// Allowed patterns (add to allowlist in tool config):
///   - Text('...stub') where 'stub' is followed by a period and description
///     (e.g., "text placeholder")
///   - Comments explaining the stub
///
/// See: https://dart.dev/analyzer/lints

import 'package:analyzer/dart/analysis/features.dart';
import 'package:analyzer/dart/analysis/utilities.dart';
import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/ast/visitor.dart';
import 'package:analyzer/error/listener.dart';

/// Finds stub UI widgets that need to be replaced with real components.
class DartStubUiFinder extends RecursiveAstVisitor<void> {
  final List<String> allowlist = [
    // Allow stubs that are clearly placeholders with descriptive text
    r'Text\([^)]*\("stub"\)',
    r'Center\(child: Text\([^)]*"stub"[^)]*\)\)',
    r'\$Text\([^)]*"stub"\)',
  ];

  DartStubUiFinder(this._listener);

  final ErrorCollector _listener;
  int _findCount = 0;

  @override
  void visitLiteralExpression(LiteralExpression node) {
    if (node is! StringLiteral) return;
    if (_isStub(node.text)) {
      _report(node);
    }
    super.visitLiteralExpression(node);
  }

  @override
  void visitElementPattern(VariableElementPattern node) {
    // Skip pattern variables
    super.visitElementPattern(node);
    return;
  }

  bool _isStub(String text) {
    for (final pattern in allowlist) {
      if (RegExp(pattern).hasMatch(text)) return true;
    }
    // Also check for common stub patterns
    final lower = text.toLowerCase();
    if (lower.contains('stub') || lower.contains('placeholder') ||
        lower.contains('todo') || lower.contains('fixme')) {
      return true;
    }
    return false;
  }

  void _report(AstNode node) {
    _findCount++;
    _listener.reportError(
      DiagnosticSeverity.error,
      'dart_stub_ui',
      'Found stub UI widget: "${node.toSource()}" — replace with real component.',
      node.start,
      node.end,
    );
  }

  int get findCount => _findCount;
}

void main(List<String> args) {
  final root = parseDartSource(
    '''
import 'package:flutter/material.dart';

class StubScreen extends StatelessWidget {
  const StubScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Center(child: Text('This is a stub screen')), // ❌ violation
    );
  }
}
''',
    analyzerOptions: const AnalyzerOptions(
      features: [Feature.DART_SDK],
    ),
  );

  final listener = DefaultErrorCollector.create();
  final finder = DartStubUiFinder(listener);
  final result = visitor.visit(root);

  print('Found ${finder.findCount} stub UI violations.');
  if (listener.hasErrors) {
    for (final error in listener.errors) {
      print('  $error');
    }
  }
}
