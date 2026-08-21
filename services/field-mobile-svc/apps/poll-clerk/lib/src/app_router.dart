import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'features/auth/login_screen.dart';
import 'features/station_dashboard/station_dashboard_screen.dart';
import 'features/incident/incident_screen.dart';
import 'features/eod_report/eod_report_screen.dart';
import 'features/kiosk_mode/kiosk_screen.dart';

final appRouterProvider = Provider<GoRouter>((ref) {
	return GoRouter(
		initialLocation: '/login',
		routes: [
			GoRoute(
				path: '/login',
				builder: (context, state) => const LoginScreen(),
			),
			GoRoute(
				path: '/dashboard',
				builder: (context, state) => const StationDashboardScreen(),
			),
			GoRoute(
				path: '/incident',
				builder: (context, state) => const IncidentScreen(),
			),
			GoRoute(
				path: '/eod-report',
				builder: (context, state) => const EodReportScreen(),
			),
			GoRoute(
				path: '/kiosk',
				builder: (context, state) => const KioskScreen(),
			),
		],
	);
});
