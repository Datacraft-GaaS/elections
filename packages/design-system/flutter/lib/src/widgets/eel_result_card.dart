import 'package:flutter/material.dart';

import '../theme/eel_theme.dart';
import 'eel_button.dart' show EelSeverity;

/// Signed-data display with proof status. Used for Form 34A, accreditation
/// bundles, verdicts — anything with a signature to verify (plan §5.2).
class EelResultCard extends StatelessWidget {
  const EelResultCard({
    super.key,
    required this.title,
    required this.proofStatus,
    this.rows = const [],
    this.severity,
    this.onTap,
  });

  final String title;

  /// Proof verification outcome shown as a chip (e.g. 'signature valid',
  /// 'unverified', 'mismatch').
  final String proofStatus;

  /// Severity of the proof status; defaults derived from [proofStatus] text
  /// when null.
  final EelSeverity? severity;

  /// Key/value rows of the signed payload.
  final List<({String label, String value})> rows;

  final VoidCallback? onTap;

  EelSeverity get _resolvedSeverity =>
      severity ??
      (proofStatus.toLowerCase().contains('valid') ||
              proofStatus.toLowerCase().contains('verified')
          ? EelSeverity.success
          : proofStatus.toLowerCase().contains('mismatch') ||
                  proofStatus.toLowerCase().contains('fail')
              ? EelSeverity.danger
              : EelSeverity.warning);

  @override
  Widget build(BuildContext context) {
    final c = context.eelColors;
    return Semantics(
      label: '$title, $proofStatus',
      child: Card(
        margin: EdgeInsets.zero,
        elevation: 0,
        color: c.surface,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(EelRadius.lg),
          side: BorderSide(color: c.border),
        ),
        child: InkWell(
          onTap: onTap,
          borderRadius: BorderRadius.circular(EelRadius.lg),
          child: Padding(
            padding: const EdgeInsets.all(EelSpacing.s4),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Row(
                  children: [
                    Expanded(
                      child: Text(
                        title,
                        style:
                            Theme.of(context).textTheme.titleMedium?.copyWith(
                                  color: c.textPrimary,
                                  fontWeight: FontWeight.w600,
                                ),
                      ),
                    ),
                    _ProofChip(
                        status: proofStatus, severity: _resolvedSeverity),
                  ],
                ),
                if (rows.isNotEmpty) ...[
                  const SizedBox(height: EelSpacing.s3),
                  for (final row in rows)
                    Padding(
                      padding: const EdgeInsets.only(bottom: EelSpacing.s1),
                      child: Row(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          SizedBox(
                            width: 120,
                            child: Text(
                              row.label,
                              style: TextStyle(
                                color: c.textSecondary,
                                fontSize: EelTypography.sm,
                              ),
                            ),
                          ),
                          Expanded(
                            child: Text(
                              row.value,
                              style: TextStyle(
                                color: c.textPrimary,
                                fontSize: EelTypography.sm,
                                fontFamily: EelTypography.fontFamilyMono,
                              ),
                            ),
                          ),
                        ],
                      ),
                    ),
                ],
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _ProofChip extends StatelessWidget {
  const _ProofChip({required this.status, required this.severity});

  final String status;
  final EelSeverity severity;

  @override
  Widget build(BuildContext context) {
    final c = context.eelColors;
    final (icon, color) = switch (severity) {
      EelSeverity.success => (Icons.verified_outlined, c.success),
      EelSeverity.warning => (Icons.help_outline, c.warning),
      EelSeverity.danger => (Icons.gpp_bad_outlined, c.danger),
      EelSeverity.info => (Icons.info_outline, c.info),
    };
    return Container(
      padding:
          const EdgeInsets.symmetric(horizontal: EelSpacing.s2, vertical: 2),
      decoration: BoxDecoration(
        border: Border.all(color: color),
        borderRadius: BorderRadius.circular(EelRadius.full),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 14, color: color),
          const SizedBox(width: 4),
          Text(status,
              style:
                  TextStyle(fontSize: EelTypography.xs, color: c.textPrimary)),
        ],
      ),
    );
  }
}
