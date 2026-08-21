import 'package:flutter/material.dart';

class SlaTimerChip extends StatelessWidget {
  final DateTime slaTargetAt;

  const SlaTimerChip({super.key, required this.slaTargetAt});

  @override
  Widget build(BuildContext context) {
    final remaining = slaTargetAt.difference(DateTime.now());
    final isBreached = remaining.isNegative;
    return Chip(
      backgroundColor: isBreached ? Colors.red : Colors.green,
      label: Text(
        isBreached
            ? 'SLA BREACHED'
            : '${remaining.inMinutes}m remaining',
        style: const TextStyle(color: Colors.white, fontSize: 12),
      ),
    );
  }
}
