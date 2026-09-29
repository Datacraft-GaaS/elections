import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:eel_core_flutter/eel_core_flutter.dart';
import 'src/app.dart';
import 'src/bootstrap.dart';

void main() {
  runApp(
    const ProviderScope(
      child: _BootstrapGate(child: CvrFieldOfficerApp()),
    ),
  );
}

/// Gates the app behind the startup FSM (plan §2.4). On ready the app shows;
/// on a blocked terminal state a support-code screen is shown instead.
class _BootstrapGate extends StatefulWidget {
  const _BootstrapGate({required this.child});

  final Widget child;

  @override
  State<_BootstrapGate> createState() => _BootstrapGateState();
}

class _BootstrapGateState extends State<_BootstrapGate> {
  BootstrapState? _state;

  @override
  void initState() {
    super.initState();
    CvrBootstrap.run().then((state) {
      if (!mounted) return;
      setState(() => _state = state);
    });
  }

  @override
  Widget build(BuildContext context) {
    final s = _state;
    if (s == null || !s.isReady && !s.isBlocked) {
      return const Center(child: CircularProgressIndicator());
    }
    if (s.isBlocked) {
      return Scaffold(
        body: Center(
          child: Text('Support code: ${s.supportCode ?? 'CVR-BOOT-01'}'),
        ),
      );
    }
    return widget.child;
  }
}
