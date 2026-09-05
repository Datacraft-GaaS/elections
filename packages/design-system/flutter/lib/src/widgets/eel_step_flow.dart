import 'package:flutter/material.dart';

import '../theme/eel_theme.dart';

/// Numbered wizard with resume support. Steps persist their completion so a
/// killed app can resume at the last step (plan §5.2).
class EelStepFlow extends StatelessWidget {
  const EelStepFlow({
    super.key,
    required this.steps,
    required this.currentStep,
    required this.onStepTapped,
    required this.child,
    this.onNext,
    this.onBack,
    this.nextLabel = 'Next',
    this.backLabel = 'Back',
    this.finishLabel = 'Finish',
  });

  final List<String> steps;
  final int currentStep;
  final ValueChanged<int> onStepTapped;
  final Widget child;
  final VoidCallback? onNext;
  final VoidCallback? onBack;
  final String nextLabel;
  final String backLabel;
  final String finishLabel;

  bool get _isLast => currentStep == steps.length - 1;

  @override
  Widget build(BuildContext context) {
    final c = context.eelColors;
    return Column(
      children: [
        // Step rail — numbered, clickable for completed steps only (resume).
        Semantics(
          label: 'Step ${currentStep + 1} of ${steps.length}',
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: EelSpacing.s2),
            child: Row(
              children: [
                for (var i = 0; i < steps.length; i++) ...[
                  _StepDot(
                    index: i,
                    label: steps[i],
                    state: i < currentStep
                        ? _StepState.done
                        : i == currentStep
                            ? _StepState.current
                            : _StepState.upcoming,
                    onTap: i <= currentStep ? () => onStepTapped(i) : null,
                  ),
                  if (i < steps.length - 1)
                    Expanded(
                      child: Container(height: 2, color: c.border),
                    ),
                ],
              ],
            ),
          ),
        ),
        Expanded(child: child),
        // Bottom action bar in thumb arc (plan §5.1).
        Padding(
          padding: const EdgeInsets.all(EelSpacing.s4),
          child: Row(
            children: [
              if (currentStep > 0)
                OutlinedButton(
                  onPressed: onBack,
                  style: OutlinedButton.styleFrom(
                    minimumSize: const Size(0, EelSizing.enforcedTouch),
                  ),
                  child: Text(backLabel),
                ),
              const Spacer(),
              FilledButton(
                onPressed: onNext,
                style: FilledButton.styleFrom(
                  minimumSize: const Size(0, EelSizing.enforcedTouch),
                ),
                child: Text(_isLast ? finishLabel : nextLabel),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

enum _StepState { done, current, upcoming }

class _StepDot extends StatelessWidget {
  const _StepDot({
    required this.index,
    required this.label,
    required this.state,
    this.onTap,
  });

  final int index;
  final String label;
  final _StepState state;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final c = context.eelColors;
    final color = switch (state) {
      _StepState.done => c.success,
      _StepState.current => c.brandPrimary,
      _StepState.upcoming => c.textDisabled,
    };
    final child = Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: EelSizing.enforcedTouch / 2,
          height: EelSizing.enforcedTouch / 2,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            color: state == _StepState.upcoming ? c.background : color,
            border: Border.all(color: color, width: 2),
          ),
          alignment: Alignment.center,
          child: state == _StepState.done
              ? Icon(Icons.check, size: 16, color: c.background)
              : Text(
                  '${index + 1}',
                  style: TextStyle(
                    color: state == _StepState.upcoming
                        ? c.textDisabled
                        : c.background,
                    fontWeight: FontWeight.w600,
                  ),
                ),
        ),
        const SizedBox(height: EelSpacing.s1),
        Text(
          label,
          style: TextStyle(
            fontSize: EelTypography.xs,
            color:
                state == _StepState.current ? c.textPrimary : c.textSecondary,
            fontWeight:
                state == _StepState.current ? FontWeight.w600 : FontWeight.w400,
          ),
        ),
      ],
    );
    return onTap == null ? child : InkWell(onTap: onTap, child: child);
  }
}
