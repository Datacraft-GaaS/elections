import 'package:flutter/material.dart';

/// Minimum hit target wrapper enforcing >= 48 dp per WCAG 2.5.5.
class EelMinHitTarget extends StatelessWidget {
  const EelMinHitTarget({
    super.key,
    required this.child,
    this.onTap,
    this.semanticsLabel,
    this.minSize = 48.0,
  });

  final Widget child;
  final VoidCallback? onTap;
  final String? semanticsLabel;
  final double minSize;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      label: semanticsLabel,
      button: onTap != null,
      child: GestureDetector(
        onTap: onTap,
        child: ConstrainedBox(
          constraints: BoxConstraints(minWidth: minSize, minHeight: minSize),
          child: Center(child: child),
        ),
      ),
    );
  }
}

/// Semantics wrapper for non-interactive elements.
class EelSemanticsLabel extends StatelessWidget {
  const EelSemanticsLabel({
    super.key,
    required this.label,
    required this.child,
  });

  final String label;
  final Widget child;

  @override
  Widget build(BuildContext context) => Semantics(label: label, child: child);
}
