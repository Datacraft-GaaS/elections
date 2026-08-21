import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'auth_provider.dart';

class LoginScreen extends ConsumerStatefulWidget {
	const LoginScreen({super.key});

	@override
	ConsumerState<LoginScreen> createState() => _LoginScreenState();
}

class _LoginScreenState extends ConsumerState<LoginScreen> {
	final _clerkIdCtrl = TextEditingController();
	final _pinCtrl = TextEditingController();

	@override
	void dispose() {
		_clerkIdCtrl.dispose();
		_pinCtrl.dispose();
		super.dispose();
	}

	Future<void> _submit() async {
		await ref.read(authProvider.notifier).login(
			_clerkIdCtrl.text.trim(),
			_pinCtrl.text,
		);
		if (mounted && ref.read(authProvider).status == AuthStatus.authenticated) {
			context.go('/dashboard');
		}
	}

	@override
	Widget build(BuildContext context) {
		final auth = ref.watch(authProvider);
		return Scaffold(
			appBar: AppBar(title: const Text('Poll Clerk Login')),
			body: Padding(
				padding: const EdgeInsets.all(24),
				child: Column(
					mainAxisAlignment: MainAxisAlignment.center,
					children: [
						TextField(controller: _clerkIdCtrl, decoration: const InputDecoration(labelText: 'Clerk ID')),
						const SizedBox(height: 16),
						TextField(controller: _pinCtrl, obscureText: true, decoration: const InputDecoration(labelText: 'PIN')),
						const SizedBox(height: 24),
						if (auth.status == AuthStatus.loading)
							const CircularProgressIndicator()
						else
							ElevatedButton(onPressed: _submit, child: const Text('Login')),
					],
				),
			),
		);
	}
}
