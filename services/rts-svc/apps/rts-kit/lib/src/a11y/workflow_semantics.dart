/// Accessibility helpers specific to the RTS workflow.
///
/// Each workflow step must annotate its key actions and live regions per
/// WCAG 2.5.5 (min hit target) and WCAG 4.1.3 (status messages).
library;

import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:flutter/material.dart';

export 'package:eel_kit_sdk/eel_kit_sdk.dart' show EelMinHitTarget, EelSemanticsLabel;

/// Wraps a banner widget with a semantic live region announcement.
///
/// [role] is one of 'error' | 'warning' | 'instruction' | 'status'.
class WfSemanticsBanner extends StatelessWidget {
  const WfSemanticsBanner({
    super.key,
    required this.message,
    required this.role,
    required this.child,
  });

  final String message;
  final String role;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      liveRegion: true,
      label: '$role: $message',
      child: child,
    );
  }
}

/// Step indicator with screen-reader hint for multi-step workflows.
class WfStepIndicator extends StatelessWidget {
  const WfStepIndicator({
    super.key,
    required this.current,
    required this.total,
    required this.label,
  });

  final int current;
  final int total;
  final String label;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      label: 'Step $current of $total: $label',
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 8),
        child: Row(
          children: [
            Text(
              'Step $current / $total',
              style: Theme.of(context).textTheme.labelSmall,
            ),
            const SizedBox(width: 8),
            Expanded(
              child: LinearProgressIndicator(
                value: current / total,
                semanticsLabel: 'Workflow progress',
                semanticsValue: '$current of $total steps complete',
              ),
            ),
            const SizedBox(width: 8),
            Text(label, style: Theme.of(context).textTheme.labelSmall),
          ],
        ),
      ),
    );
  }
}

/// Confirm dialog with accessibility-labelled action buttons.
Future<bool> showWfConfirmDialog({
  required BuildContext context,
  required String title,
  required String body,
  String confirmLabel = 'Confirm',
  String cancelLabel = 'Cancel',
}) async {
  final result = await showDialog<bool>(
    context: context,
    builder: (ctx) => AlertDialog(
      title: Text(title),
      content: Text(body),
      actions: [
        EelMinHitTarget(
          semanticsLabel: cancelLabel,
          onTap: () => Navigator.of(ctx).pop(false),
          child: TextButton(
            onPressed: () => Navigator.of(ctx).pop(false),
            child: Text(cancelLabel),
          ),
        ),
        EelMinHitTarget(
          semanticsLabel: confirmLabel,
          onTap: () => Navigator.of(ctx).pop(true),
          child: FilledButton(
            onPressed: () => Navigator.of(ctx).pop(true),
            child: Text(confirmLabel),
          ),
        ),
      ],
    ),
  );
  return result ?? false;
}
