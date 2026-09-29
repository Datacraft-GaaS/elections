import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'features/auth/login_screen.dart';
import 'features/station_dashboard/station_dashboard_screen.dart';
import 'features/incident/incident_screen.dart';
import 'features/eod_report/eod_report_screen.dart';
import 'features/kiosk_mode/kiosk_screen.dart';
import 'features/queue/queue_screen.dart';
import 'features/voter_lookup/voter_lookup_screen.dart';
import 'features/reference_library/reference_library_screen.dart';

/// poll-clerk routes (M31):
/// /login → /dashboard → /incident, /queue, /voter-lookup, /eod-report, /reference-library, /kiosk
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
				path: '/queue',
				builder: (context, state) => const QueueScreen(),
			),
			GoRoute(
				path: '/voter-lookup',
				builder: (context, state) => const VoterLookupScreen(),
			),
			GoRoute(
				path: '/reference-library',
				builder: (context, state) => const ReferenceLibraryScreen(),
			),
			GoRoute(
				path: '/eod-report',
				builder: (context, state) => const EODReportScreen(),
			),
			GoRoute(
				path: '/kiosk',
				builder: (context, state) => const KioskScreen(),
			),
		],
	);
});
