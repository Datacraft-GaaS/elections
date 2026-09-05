/// M3 feature-screen gate — wraps any feature widget with hardware-availability banner.
/// Per plan.md M3: feature screens must show "hardware required" message when devices missing.
library;

import 'package:flutter/material.dart';
import '../session/m3_session.dart';

class M3FeatureGate extends StatelessWidget {
  final M3SessionState session;
  final Widget child;
  const M3FeatureGate({super.key, required this.session, required this.child});

  @override
  Widget build(BuildContext context) {
    if (session.isReady) return child;
    return Column(
      children: [
        Container(
          width: double.infinity,
          color: Theme.of(context).colorScheme.errorContainer,
          padding: const EdgeInsets.all(12),
          child: Text(
            session.errorMessage ?? 'Hardware required (M3)',
            style: TextStyle(color: Theme.of(context).colorScheme.onErrorContainer),
          ),
        ),
        const Expanded(child: Center(child: Text('Feature disabled — hardware not procured.'))),
      ],
    );
  }
}
