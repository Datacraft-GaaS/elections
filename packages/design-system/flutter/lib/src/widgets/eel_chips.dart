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
import 'eel_button.dart' show EelSeverity;

/// Severity chip: status / connectivity / outcome at a glance. Never uses
/// colour alone — carries an icon + label (WCAG 2.2, plan §5.1).
class EelStatusChip extends StatelessWidget {
  const EelStatusChip({
    super.key,
    required this.label,
    this.severity = EelSeverity.info,
    this.icon,
  });

  final String label;
  final EelSeverity severity;
  final IconData? icon;

  @override
  Widget build(BuildContext context) {
    final c = context.eelColors;
    final (iconData, color) = switch (severity) {
      EelSeverity.info => (Icons.info_outline, c.info),
      EelSeverity.success => (Icons.check_circle_outline, c.success),
      EelSeverity.warning => (Icons.warning_amber_outlined, c.warning),
      EelSeverity.danger => (Icons.error_outline, c.danger),
    };
    return Semantics(
      label: '$label, ${severity.name}',
      child: Container(
        padding: const EdgeInsets.symmetric(
          horizontal: EelSpacing.s2,
          vertical: EelSpacing.s1,
        ),
        decoration: BoxDecoration(
          color: c.surface,
          border: Border.all(color: color, width: 1),
          borderRadius: BorderRadius.circular(EelRadius.full),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon ?? iconData, size: 16, color: color),
            const SizedBox(width: EelSpacing.s1),
            Text(
              label,
              style:
                  TextStyle(color: c.textPrimary, fontSize: EelTypography.sm),
            ),
          ],
        ),
      ),
    );
  }
}

/// SLA countdown chip: remaining time + phase; colour escalates but text
/// always states the phase so colour is never the only signal.
class EelSlaTimerChip extends StatefulWidget {
  const EelSlaTimerChip({
    super.key,
    required this.deadline,
    required this.phaseLabels,
    this.now,
  });

  /// When the SLA expires.
  final DateTime deadline;

  /// Labels by remaining-fraction buckets: {normal, warning, critical}.
  final ({String normal, String warning, String critical}) phaseLabels;

  /// Injectable clock for tests.
  final DateTime Function()? now;

  @override
  State<EelSlaTimerChip> createState() => _EelSlaTimerChipState();
}

class _EelSlaTimerChipState extends State<EelSlaTimerChip> {
  @override
  Widget build(BuildContext context) {
    final now = (widget.now ?? DateTime.now)();
    final remaining = widget.deadline.difference(now);
    const total = Duration(minutes: 30);
    final fraction = remaining.inMilliseconds / total.inMilliseconds;
    final (label, severity) = fraction > 0.33
        ? (widget.phaseLabels.normal, EelSeverity.success)
        : fraction > 0.10
            ? (widget.phaseLabels.warning, EelSeverity.warning)
            : (widget.phaseLabels.critical, EelSeverity.danger);
    final mm = remaining.inMinutes.remainder(60).toString().padLeft(2, '0');
    final ss = remaining.inSeconds.remainder(60).toString().padLeft(2, '0');
    final timeLabel = remaining.isNegative ? '00:00' : '$mm:$ss';
    return EelStatusChip(
      label: '$label $timeLabel',
      severity: severity,
      icon: Icons.timer_outlined,
    );
  }
}

/// Queue depth + last-sync indicator for offline-first screens.
class EelSyncIndicator extends StatelessWidget {
  const EelSyncIndicator({
    super.key,
    required this.pendingCount,
    required this.lastSync,
    this.syncing = false,
    this.onTap,
  });

  final int pendingCount;
  final DateTime? lastSync;
  final bool syncing;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final c = context.eelColors;
    final syncLabel = lastSync == null
        ? 'never'
        : MaterialLocalizations.of(context).formatTimeOfDay(
            TimeOfDay.fromDateTime(lastSync!),
          );
    final text =
        syncing ? 'Syncing…' : '$pendingCount waiting · last sync $syncLabel';
    return Semantics(
      label: text,
      child: Material(
        type: MaterialType.transparency,
        child: InkWell(
          onTap: onTap,
          borderRadius: BorderRadius.circular(EelRadius.full),
          child: Padding(
            padding: const EdgeInsets.symmetric(
              horizontal: EelSpacing.s2,
              vertical: EelSpacing.s2,
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                syncing
                    ? const SizedBox(
                        width: 14,
                        height: 14,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : Icon(
                        pendingCount > 0
                            ? Icons.sync
                            : Icons.cloud_done_outlined,
                        size: 16,
                        color: pendingCount > 0 ? c.warning : c.success,
                      ),
                const SizedBox(width: EelSpacing.s1),
                Flexible(
                  child: Text(
                    text,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                        fontSize: EelTypography.sm, color: c.textSecondary),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

/// One selectable chip in an [EelChips] group.
class ChipOption {
  const ChipOption({required this.label, this.selected = false, this.icon});

  /// Text shown in the chip.
  final String label;

  /// Whether this option is currently selected.
  final bool selected;

  /// Optional leading glyph, for options that are not plain words.
  final IconData? icon;
}

/// A group of toggleable chips, wrapping onto as many lines as needed.
///
/// Selection state is owned by the caller: the chips report intent through
/// [onToggle] (by index, for callers holding an enum list) or [onSelected]
/// (by label, for everything else) and re-render with the new state. Nothing
/// here holds state, so the same component serves a filter bar and a
/// multi-select partition selector.
class EelChips extends StatelessWidget {
  const EelChips({
    super.key,
    required this.options,
    this.onToggle,
    this.onSelected,
    this.spacing = EelSpacing.s2,
  });

  final List<ChipOption> options;

  /// Called with the index of the tapped option.
  final ValueChanged<int>? onToggle;

  /// Called with the label of the tapped option.
  final ValueChanged<String>? onSelected;

  final double spacing;

  @override
  Widget build(BuildContext context) {
    final c = context.eelColors;
    return Wrap(
      spacing: spacing,
      runSpacing: spacing,
      children: [
        for (var i = 0; i < options.length; i++)
          Builder(builder: (context) {
            final option = options[i];
            return FilterChip(
              label: Text(option.label),
              avatar: option.icon == null ? null : Icon(option.icon, size: 18),
              selected: option.selected,
              showCheckmark: false,
              onSelected: (_) {
                onToggle?.call(i);
                onSelected?.call(option.label);
              },
              selectedColor: c.brandPrimary.withValues(alpha: 0.14),
              backgroundColor: c.surface,
              side: BorderSide(
                  color: option.selected ? c.brandPrimary : c.border),
              // Same enforced touch target as EelButton (plan §5.2).
              materialTapTargetSize: MaterialTapTargetSize.padded,
            );
          }),
      ],
    );
  }
}
