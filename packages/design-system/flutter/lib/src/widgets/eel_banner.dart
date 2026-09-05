import 'package:flutter/material.dart';

import '../theme/eel_theme.dart';
import 'eel_button.dart' show EelSeverity;

/// Info / warning / critical banner with accessible label and tone from
/// tokens `alert.*`. Never colour-only: icon + text carry the signal.
class EelBanner extends StatelessWidget {
  const EelBanner({
    super.key,
    required this.message,
    this.severity = EelSeverity.info,
    this.title,
    this.action,
    this.onDismiss,
  });

  final String message;
  final String? title;
  final EelSeverity severity;
  final Widget? action;
  final VoidCallback? onDismiss;

  @override
  Widget build(BuildContext context) {
    // Colors resolved inside EelBanner.
    final (bg, border, fg, icon, tone) = switch (severity) {
      EelSeverity.info => (
          const Color(0xFFEFF6FF),
          const Color(0xFF1A56DB),
          const Color(0xFF1E40AF),
          Icons.info_outline,
          'info'
        ),
      EelSeverity.success => (
          const Color(0xFFECFDF5),
          const Color(0xFF057A55),
          const Color(0xFF065F46),
          Icons.check_circle_outline,
          'success'
        ),
      EelSeverity.warning => (
          const Color(0xFFFFFbeb),
          const Color(0xFFD97706),
          const Color(0xFF92400E),
          Icons.warning_amber_outlined,
          'warning'
        ),
      EelSeverity.danger => (
          const Color(0xFFFEF2F2),
          const Color(0xFFDC2626),
          Color.lerp(const Color(0xFF991B1B), Colors.black, 0)!,
          Icons.error_outline,
          'critical'
        ),
    };

    return Semantics(
      label: '$tone banner: $message',
      liveRegion: severity == EelSeverity.danger,
      child: Container(
        width: double.infinity,
        padding: const EdgeInsets.all(EelSpacing.s3),
        decoration: BoxDecoration(
          color: bg,
          border: Border(left: BorderSide(color: border, width: 4)),
        ),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Icon(icon, color: border, size: 20),
            const SizedBox(width: EelSpacing.s2),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  if (title != null)
                    Text(
                      title!,
                      style: TextStyle(
                        color: fg,
                        fontWeight: FontWeight.w600,
                        fontSize: EelTypography.sm,
                      ),
                    ),
                  Text(
                    message,
                    style: TextStyle(color: fg, fontSize: EelTypography.sm),
                  ),
                ],
              ),
            ),
            if (action != null) action!,
            if (onDismiss != null)
              IconButton(
                icon: Icon(Icons.close, size: 18, color: fg),
                tooltip: 'Dismiss',
                onPressed: onDismiss,
              ),
          ],
        ),
      ),
    );
  }
}
