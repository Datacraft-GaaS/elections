import 'package:flutter/material.dart';

import '../theme/eel_theme.dart';

/// Severity for [EelButton] destructive styling and [EelBanner] tone.
enum EelSeverity { info, success, warning, danger }

/// The eel button. Variants: primary / secondary / destructive / ghost.
/// Always ≥ 48 dp tall (plan §5.2); loading state disables and shows spinner.
class EelButton extends StatelessWidget {
  const EelButton({
    super.key,
    required this.label,
    required this.onPressed,
    this.variant = EelButtonVariant.primary,
    this.loading = false,
    this.icon,
    this.expand = false,
  });

  final String label;
  final VoidCallback? onPressed;
  final EelButtonVariant variant;
  final bool loading;
  final IconData? icon;
  final bool expand;

  @override
  Widget build(BuildContext context) {
    final c = context.eelColors;
    final enabled = onPressed != null && !loading;

    final Widget child;
    switch (variant) {
      case EelButtonVariant.primary:
        child = FilledButton(
          onPressed: enabled ? onPressed : null,
          style: FilledButton.styleFrom(
            backgroundColor: c.brandPrimary,
            foregroundColor: c.textInverse,
            disabledBackgroundColor: c.textDisabled,
            minimumSize: const Size(0, EelSizing.enforcedTouch),
          ),
          child: _content(c.textInverse),
        );
      case EelButtonVariant.secondary:
        child = OutlinedButton(
          onPressed: enabled ? onPressed : null,
          style: OutlinedButton.styleFrom(
            foregroundColor: c.textPrimary,
            side: BorderSide(color: enabled ? c.border : c.textDisabled),
            minimumSize: const Size(0, EelSizing.enforcedTouch),
          ),
          child: _content(c.textPrimary),
        );
      case EelButtonVariant.destructive:
        child = FilledButton(
          onPressed: enabled ? onPressed : null,
          style: FilledButton.styleFrom(
            backgroundColor: c.danger,
            foregroundColor: c.textInverse,
            disabledBackgroundColor: c.textDisabled,
            minimumSize: const Size(0, EelSizing.enforcedTouch),
          ),
          child: _content(c.textInverse),
        );
      case EelButtonVariant.ghost:
        child = TextButton(
          onPressed: enabled ? onPressed : null,
          style: TextButton.styleFrom(
            foregroundColor: c.textLink,
            minimumSize: const Size(0, EelSizing.enforcedTouch),
          ),
          child: _content(c.textLink),
        );
    }

    return child;
  }

  Widget _content(Color fg) {
    final label_ = Text(
      loading ? '' : label,
      style: TextStyle(color: fg, fontWeight: FontWeight.w600),
    );
    if (loading) {
      return SizedBox(
        width: 20,
        height: 20,
        child: CircularProgressIndicator(strokeWidth: 2, color: fg),
      );
    }
    if (icon != null) {
      return Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 20, color: fg),
          const SizedBox(width: 8),
          label_
        ],
      );
    }
    return label_;
  }
}

enum EelButtonVariant { primary, secondary, destructive, ghost }
