import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'features/auth/login_screen.dart';
import 'features/check_in/check_in_screen.dart';
import 'features/qr_verify/qr_verify_screen.dart';
import 'features/countersign/countersign_screen.dart';
import 'features/dispute/dispute_screen.dart';

final appRouterProvider = Provider<GoRouter>((ref) {
	return GoRouter(
		initialLocation: '/login',
		routes: [
			GoRoute(
				path: '/login',
				builder: (context, state) => const LoginScreen(),
			),
			GoRoute(
				path: '/check-in',
				builder: (context, state) => const CheckInScreen(),
			),
			GoRoute(
				path: '/verify',
				builder: (context, state) => const QRVerifyScreen(),
			),
			GoRoute(
				path: '/countersign',
				builder: (context, state) => const CountersignScreen(),
			),
			GoRoute(
				path: '/dispute',
				builder: (context, state) => const DisputeScreen(),
			),
		],
	);
});
