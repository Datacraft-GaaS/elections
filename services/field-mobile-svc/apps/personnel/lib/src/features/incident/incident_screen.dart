import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class IncidentScreen extends ConsumerWidget {
  const IncidentScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Scaffold(
      appBar: AppBar(title: const Text('Incident Report')),
      body: const Center(child: Text('Submit incident report')),
    );
  }
}
