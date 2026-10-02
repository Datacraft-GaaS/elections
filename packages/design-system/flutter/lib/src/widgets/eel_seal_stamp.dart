// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import 'package:flutter/material.dart';

import '../theme/eel_theme.dart';

/// Seal stamp: displays a physical seal / container seal id with verification
/// state. Ported from field-support (plan §5.2).
class EelSealStamp extends StatelessWidget {
  const EelSealStamp({
    super.key,
    this.sealId,
    this.verified,
    this.verifiedAt,
    this.onVerify,
    this.size,
    this.color,
    this.label,
  });

  /// The container seal id. Null renders the compact emblem form.
  final String? sealId;

  /// Verification state. Null is treated as not verified; a seal with no id
  /// cannot be verified, and saying so is the honest reading.
  final bool? verified;

  final DateTime? verifiedAt;
  final VoidCallback? onVerify;

  /// Emblem edge length, for the compact form.
  final double? size;

  /// Emblem colour, for the compact form. Defaults to the verification colour.
  final Color? color;

  /// Wordmark shown beside the emblem in the compact form.
  final String? label;

  @override
  Widget build(BuildContext context) {
    final c = context.eelColors;
    if (sealId == null) return _emblem(context, c);
    final isVerified = verified ?? false;
    final color = isVerified ? c.success : c.warning;
    return Semantics(
      label: 'Seal $sealId, ${isVerified ? 'verified' : 'not verified'}',
      child: Container(
        padding: const EdgeInsets.all(EelSpacing.s3),
        decoration: BoxDecoration(
          border: Border.all(color: color, width: 2),
          borderRadius: BorderRadius.circular(EelRadius.lg),
        ),
        child: Row(
          children: [
            Icon(
              isVerified ? Icons.verified_user_outlined : Icons.shield_outlined,
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
                    sealId!,
                    style: TextStyle(
                      color: c.textPrimary,
                      fontFamily: EelTypography.fontFamilyMono,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                  Text(
                    isVerified ? 'Seal verified' : 'Seal not verified',
                    style: TextStyle(
                        color: c.textSecondary, fontSize: EelTypography.sm),
                  ),
                ],
              ),
            ),
            if (!isVerified && onVerify != null)
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

  /// The compact form: just the mark, for use as a card's leading emblem.
  ///
  /// Carries no id and no verification claim. The alternative was a separate
  /// widget, but every call site was already passing size and colour to this
  /// one, so one component with two honest forms beat five rewrites.
  Widget _emblem(BuildContext context, EelColors c) {
    final isVerified = verified ?? false;
    final tint = color ?? (isVerified ? c.success : c.warning);
    final edge = size ?? 48.0;
    return Semantics(
      label: isVerified ? 'Seal, verified' : 'Seal',
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            isVerified ? Icons.verified_user_outlined : Icons.shield_outlined,
            color: tint,
            size: edge,
          ),
          if (label != null) ...[
            const SizedBox(width: EelSpacing.s3),
            Text(
              label!,
              style: TextStyle(
                color: c.textPrimary,
                fontSize: edge * 0.4,
                fontWeight: FontWeight.w600,
              ),
            ),
          ],
        ],
      ),
    );
  }
}
