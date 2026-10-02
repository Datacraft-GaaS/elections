// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
