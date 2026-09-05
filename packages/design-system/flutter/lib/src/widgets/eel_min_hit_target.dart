import 'package:flutter/material.dart';

/// Re-export of kit-sdk's hit-target wrapper so apps import a single package.
/// Local copy so eel_design stays dependency-free; kit-sdk re-exports this.
/// Enforces ≥ [minSize] (48 dp default) per WCAG 2.5.5 / plan §5.1.
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
