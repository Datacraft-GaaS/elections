import 'package:flutter/material.dart';

import '../theme/eel_theme.dart';

/// Seal stamp: displays a physical seal / container seal id with verification
/// state. Ported from field-support (plan §5.2).
class EelSealStamp extends StatelessWidget {
  const EelSealStamp({
    super.key,
    required this.sealId,
    required this.verified,
    this.verifiedAt,
    this.onVerify,
  });

  final String sealId;
  final bool verified;
  final DateTime? verifiedAt;
  final VoidCallback? onVerify;

  @override
  Widget build(BuildContext context) {
    final c = context.eelColors;
    final color = verified ? c.success : c.warning;
    return Semantics(
      label: 'Seal $sealId, ${verified ? 'verified' : 'not verified'}',
      child: Container(
        padding: const EdgeInsets.all(EelSpacing.s3),
        decoration: BoxDecoration(
          border: Border.all(color: color, width: 2),
          borderRadius: BorderRadius.circular(EelRadius.lg),
        ),
        child: Row(
          children: [
            Icon(
              verified ? Icons.verified_user_outlined : Icons.shield_outlined,
              color: color,
              size: 28,
            ),
            const SizedBox(width: EelSpacing.s3),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(
                    sealId,
                    style: TextStyle(
                      color: c.textPrimary,
                      fontFamily: EelTypography.fontFamilyMono,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                  Text(
                    verified ? 'Seal verified' : 'Seal not verified',
                    style: TextStyle(
                        color: c.textSecondary, fontSize: EelTypography.sm),
                  ),
                ],
              ),
            ),
            if (!verified && onVerify != null)
              TextButton(
                onPressed: onVerify,
                style: TextButton.styleFrom(
                  minimumSize: const Size(0, EelSizing.enforcedTouch),
                ),
                child: const Text('Verify'),
              ),
          ],
        ),
      ),
    );
  }
}
