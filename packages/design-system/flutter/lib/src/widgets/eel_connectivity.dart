import 'package:flutter/material.dart';

import 'eel_banner.dart';
import 'eel_button.dart' show EelSeverity;
import 'eel_chips.dart';

/// Connectivity banner states (plan §2.6).
enum EelConnectivityState {
  online,
  degraded,
  offline,
  captivePortal,
  untrusted
}

/// Connectivity banner shown at top of screens when not online.
class EelConnectivityBanner extends StatelessWidget {
  const EelConnectivityBanner({
    super.key,
    required this.state,
    this.onRetry,
    this.onDetails,
  });

  final EelConnectivityState state;
  final VoidCallback? onRetry;
  final VoidCallback? onDetails;

  @override
  Widget build(BuildContext context) {
    switch (state) {
      case EelConnectivityState.online:
        return const SizedBox.shrink();
      case EelConnectivityState.degraded:
        return const EelBanner(
          severity: EelSeverity.warning,
          title: 'Limited connectivity',
          message: 'Connection is slow. Large uploads will ask for consent.',
        );
      case EelConnectivityState.offline:
        return const EelBanner(
          severity: EelSeverity.info,
          title: 'Offline',
          message:
              'Work continues offline. Changes will sync when reconnected.',
        );
      case EelConnectivityState.captivePortal:
        return EelBanner(
          severity: EelSeverity.warning,
          title: 'Network blocked',
          message:
              'A sign-in page is blocking the connection. Open a browser to complete sign-in.',
          action: onDetails == null
              ? null
              : TextButton(
                  onPressed: onDetails,
                  child: const Text('Details'),
                ),
        );
      case EelConnectivityState.untrusted:
        return EelBanner(
          severity: EelSeverity.danger,
          title: 'Network blocked',
          message:
              'The server could not be verified. For safety, sync is disabled.',
          action: onRetry == null
              ? null
              : TextButton(
                  onPressed: onRetry,
                  child: const Text('Retry'),
                ),
        );
    }
  }
}

/// Connectivity status pill used inside toolbars.
class EelConnectivityChip extends StatelessWidget {
  const EelConnectivityChip({super.key, required this.state});

  final EelConnectivityState state;

  @override
  Widget build(BuildContext context) {
    final (label, severity) = switch (state) {
      EelConnectivityState.online => ('Online', EelSeverity.success),
      EelConnectivityState.degraded => ('Slow network', EelSeverity.warning),
      EelConnectivityState.offline => ('Offline', EelSeverity.info),
      EelConnectivityState.captivePortal => (
          'Sign-in needed',
          EelSeverity.warning
        ),
      EelConnectivityState.untrusted => ('Blocked', EelSeverity.danger),
    };
    return EelStatusChip(label: label, severity: severity);
  }
}
