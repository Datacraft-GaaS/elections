import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class StatusPingScreen extends ConsumerWidget {
  const StatusPingScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Scaffold(
      appBar: AppBar(title: const Text('Status Ping')),
      body: const Center(child: Text('15-min status ping')),
    );
  }
}
