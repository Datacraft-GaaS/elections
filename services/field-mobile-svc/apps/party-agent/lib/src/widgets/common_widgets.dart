import 'package:flutter/material.dart';

class EelLoadingIndicator extends StatelessWidget {
	const EelLoadingIndicator({super.key});

	@override
	Widget build(BuildContext context) {
		return const Center(child: CircularProgressIndicator());
	}
}

class EelErrorDisplay extends StatelessWidget {
	final String message;
	final VoidCallback? onRetry;

	const EelErrorDisplay({super.key, required this.message, this.onRetry});

	@override
	Widget build(BuildContext context) {
		return Center(
			child: Column(
				mainAxisSize: MainAxisSize.min,
				children: [
					Text(message, style: TextStyle(color: Theme.of(context).colorScheme.error)),
					if (onRetry != null) ...[
						const SizedBox(height: 12),
						ElevatedButton(onPressed: onRetry, child: const Text('Retry')),
					],
				],
			),
		);
	}
}

class EelPrimaryButton extends StatelessWidget {
	final String label;
	final VoidCallback? onPressed;

	const EelPrimaryButton({super.key, required this.label, this.onPressed});

	@override
	Widget build(BuildContext context) {
		return SizedBox(
			width: double.infinity,
			child: ElevatedButton(onPressed: onPressed, child: Text(label)),
		);
	}
}
