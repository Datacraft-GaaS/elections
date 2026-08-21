import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class DiagnosticsScreen extends ConsumerWidget {
  final String kitSerial;
  const DiagnosticsScreen({super.key, required this.kitSerial});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Scaffold(
      appBar: AppBar(title: Text('Diagnostics: $kitSerial')),
      body: const Center(child: Text('Remote kit diagnostics')),
    );
  }
}
