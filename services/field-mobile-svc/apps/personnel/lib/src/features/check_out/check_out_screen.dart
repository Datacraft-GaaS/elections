import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class CheckOutScreen extends ConsumerWidget {
  const CheckOutScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Scaffold(
      appBar: AppBar(title: const Text('GPS Check-Out')),
      body: const Center(child: Text('GPS-verified check-out with payroll trigger stub')),
    );
  }
}
