import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class WarrantyScreen extends ConsumerWidget {
  const WarrantyScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Scaffold(
      appBar: AppBar(title: const Text('KEBS Warranty Claims')),
      body: const Center(child: Text('Warranty claim initiation')),
    );
  }
}
