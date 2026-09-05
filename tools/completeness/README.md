# Completeness Scanner

A tool that scans the codebase for missing pieces and enforces completeness
gates in CI. It uses two main rules:

## Dart Stub UI Detection (`dart_stub_ui.dart`)

Detects stub UI widgets like `Text('stub')` or `Center(child: Text('...stub'))`
that should be replaced with real components.

```bash
# Run on a single file
dart analyze --rules=tools/completeness/rules/dart_stub_ui.dart \
  services/field-mobile-svc/apps/poll-clerk/lib/src/features/login/screen.dart

# Or integrate into your existing analyzer setup:
# Add to analysis_options.yaml:
#   linter:
#     rules:
#       - dart_stub_ui
```

## Usage in CI

Add this to `.woodpecker/test.yaml`:

```yaml
steps:
  - name: completeness-check
    image: python:3.13-slim
    commands:
      - pip install pylint flake8
      - tools/completeness/check.py
```

## Adding New Rules

Create a file in `rules/` with `.py` extension following the same pattern as
the Dart rule. The tool automatically discovers all `.dart`, `.rs`, `.py` rule
files and runs them in parallel.

## Rule Format

Each rule should implement:

1. A class that visits AST nodes (or equivalent for Rust)
2. A `findCount` counter for CI gating
3. Reports violations with line/column info

Example structure:

```python
class MyRuleVisitor(AstVisitor):
    def visit_FunctionDefinition(self, node):
        if self._has_todo(node):
            self.report(node)
```

## Integration with justfile

```bash
just completeness-check  # runs all rules in parallel
```

## Current Rules

| Rule | Language | Purpose | Status |
|------|----------|---------|--------|
| `dart_stub_ui` | Dart | Detects placeholder UI widgets | ✅ Created |
| (future) `rust_unimplemented` | Rust | Detects `unimplemented!()` or `todo!()` macros | ⬜ TODO |
