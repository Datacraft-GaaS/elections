import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

class CheckInScreen extends ConsumerWidget {
	const CheckInScreen({super.key});

	@override
	Widget build(BuildContext context, WidgetRef ref) {
		return Scaffold(
			appBar: AppBar(title: const Text('Check In')),
			body: const Center(
				child: Text('GPS + biometric check-in stub'),
			),
		);
	}
}
