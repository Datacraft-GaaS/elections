import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class ShiftReportScreen extends ConsumerWidget {
  const ShiftReportScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Scaffold(
      appBar: AppBar(title: const Text('EOD Shift Report')),
      body: const Center(child: Text('End of day signed report')),
    );
  }
}
