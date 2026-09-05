import 'package:flutter/material.dart';

import '../theme/eel_theme.dart';

/// Full-screen empty state: icon, title, body, optional action.
class EelEmptyState extends StatelessWidget {
  const EelEmptyState({
    super.key,
    required this.title,
    this.message,
    this.icon = Icons.inbox_outlined,
    this.actionLabel,
    this.onAction,
  });

  final String title;
  final String? message;
  final IconData icon;
  final String? actionLabel;
  final VoidCallback? onAction;

  @override
  Widget build(BuildContext context) {
    final c = context.eelColors;
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(EelSpacing.s6),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, size: 48, color: c.textDisabled),
            const SizedBox(height: EelSpacing.s4),
            Text(
              title,
              style: Theme.of(context)
                  .textTheme
                  .titleLarge
                  ?.copyWith(color: c.textPrimary),
              textAlign: TextAlign.center,
            ),
            if (message != null) ...[
              const SizedBox(height: EelSpacing.s2),
              Text(
                message!,
                style: TextStyle(
                    color: c.textSecondary, fontSize: EelTypography.sm),
                textAlign: TextAlign.center,
              ),
            ],
            if (actionLabel != null && onAction != null) ...[
              const SizedBox(height: EelSpacing.s4),
              FilledButton(
                onPressed: onAction,
                style: FilledButton.styleFrom(
                  minimumSize: const Size(0, EelSizing.enforcedTouch),
                ),
                child: Text(actionLabel!),
              ),
            ],
          ],
        ),
      ),
    );
  }
}

/// Loading state with accessible progress announcement.
class EelLoadingState extends StatelessWidget {
  const EelLoadingState({super.key, this.message = 'Loading…'});

  final String message;

  @override
  Widget build(BuildContext context) {
    final c = context.eelColors;
    return Semantics(
      label: message,
      child: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const CircularProgressIndicator(),
            const SizedBox(height: EelSpacing.s3),
            Text(
              message,
              style:
                  TextStyle(color: c.textSecondary, fontSize: EelTypography.sm),
            ),
          ],
        ),
      ),
    );
  }
}

/// Error state parsed from an RFC 9457 problem (`type`, `title`, `status`,
/// `detail`, `instance`). Shows what happened + what to do now + error id
/// for support (plan §5.1).
class EelErrorState extends StatelessWidget {
  const EelErrorState({
    super.key,
    required this.problem,
    this.onRetry,
  });

  final EelProblem problem;
  final VoidCallback? onRetry;

  @override
  Widget build(BuildContext context) {
    final c = context.eelColors;
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(EelSpacing.s6),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.error_outline, size: 48, color: c.danger),
            const SizedBox(height: EelSpacing.s4),
            Text(
              problem.title,
              style: Theme.of(context)
                  .textTheme
                  .titleLarge
                  ?.copyWith(color: c.textPrimary),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: EelSpacing.s2),
            Text(
              problem.detail ?? 'An unexpected error occurred.',
              style:
                  TextStyle(color: c.textSecondary, fontSize: EelTypography.sm),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: EelSpacing.s2),
            Text(
              'Error ${problem.status} · ${problem.instance}',
              style:
                  TextStyle(color: c.textDisabled, fontSize: EelTypography.xs),
              textAlign: TextAlign.center,
            ),
            if (onRetry != null) ...[
              const SizedBox(height: EelSpacing.s4),
              OutlinedButton(
                onPressed: onRetry,
                style: OutlinedButton.styleFrom(
                  minimumSize: const Size(0, EelSizing.enforcedTouch),
                ),
                child: const Text('Try again'),
              ),
            ],
          ],
        ),
      ),
    );
  }
}

/// Offline state: offline is normal, not error (plan §5.1).
class EelOfflineState extends StatelessWidget {
  const EelOfflineState({
    super.key,
    required this.title,
    this.message,
    this.actionLabel,
    this.onAction,
  });

  final String title;
  final String? message;
  final String? actionLabel;
  final VoidCallback? onAction;

  @override
  Widget build(BuildContext context) {
    final c = context.eelColors;
    return Semantics(
      label: 'Offline: $title',
      child: Center(
        child: Padding(
          padding: const EdgeInsets.all(EelSpacing.s6),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(Icons.cloud_off_outlined, size: 48, color: c.textDisabled),
              const SizedBox(height: EelSpacing.s4),
              Text(
                title,
                style: Theme.of(context)
                    .textTheme
                    .titleLarge
                    ?.copyWith(color: c.textPrimary),
                textAlign: TextAlign.center,
              ),
              if (message != null) ...[
                const SizedBox(height: EelSpacing.s2),
                Text(
                  message!,
                  style: TextStyle(
                      color: c.textSecondary, fontSize: EelTypography.sm),
                  textAlign: TextAlign.center,
                ),
              ],
              if (actionLabel != null && onAction != null) ...[
                const SizedBox(height: EelSpacing.s4),
                OutlinedButton(
                  onPressed: onAction,
                  style: OutlinedButton.styleFrom(
                    minimumSize: const Size(0, EelSizing.enforcedTouch),
                  ),
                  child: Text(actionLabel!),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}

/// Permission rationale screen shown before OS permission prompt.
class EelPermissionRationale extends StatelessWidget {
  const EelPermissionRationale({
    super.key,
    required this.title,
    required this.rationale,
    required this.continueLabel,
    required this.onContinue,
    this.icon = Icons.lock_outline,
  });

  final String title;
  final String rationale;
  final String continueLabel;
  final VoidCallback onContinue;
  final IconData icon;

  @override
  Widget build(BuildContext context) {
    final c = context.eelColors;
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(EelSpacing.s6),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, size: 48, color: c.brandPrimary),
            const SizedBox(height: EelSpacing.s4),
            Text(
              title,
              style: Theme.of(context)
                  .textTheme
                  .titleLarge
                  ?.copyWith(color: c.textPrimary),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: EelSpacing.s2),
            Text(
              rationale,
              style:
                  TextStyle(color: c.textSecondary, fontSize: EelTypography.sm),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: EelSpacing.s4),
            FilledButton(
              onPressed: onContinue,
              style: FilledButton.styleFrom(
                minimumSize: const Size(0, EelSizing.enforcedTouch),
              ),
              child: Text(continueLabel),
            ),
          ],
        ),
      ),
    );
  }
}

/// RFC 9457 problem details, carried into [EelErrorState]. App-side parsing
/// lives in eel-core-flutter; this is the display model.
class EelProblem {
  const EelProblem({
    required this.type,
    required this.title,
    required this.status,
    this.detail,
    this.instance,
  });

  final String type;
  final String title;
  final int status;
  final String? detail;
  final String? instance;

  factory EelProblem.fromJson(Map<String, dynamic> json) => EelProblem(
        type: json['type'] as String? ?? 'about:blank',
        title: json['title'] as String? ?? 'Error',
        status: json['status'] as int? ?? 500,
        detail: json['detail'] as String?,
        instance: json['instance'] as String?,
      );
}
