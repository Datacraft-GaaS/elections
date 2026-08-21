import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'features/auth/login_screen.dart';
import 'features/station_visit/station_visit_screen.dart';
import 'features/incident/incident_screen.dart';
import 'features/tally_verify/tally_verify_screen.dart';
import 'features/mission_export/mission_export_screen.dart';

final appRouterProvider = Provider<GoRouter>((ref) {
	return GoRouter(
		initialLocation: '/login',
		routes: [
			GoRoute(
				path: '/login',
				builder: (context, state) => const LoginScreen(),
			),
			GoRoute(
				path: '/station-visit',
				builder: (context, state) => const StationVisitScreen(),
			),
			GoRoute(
				path: '/incident',
				builder: (context, state) => const IncidentScreen(),
			),
			GoRoute(
				path: '/tally-verify',
				builder: (context, state) => const TallyVerifyScreen(),
			),
			GoRoute(
				path: '/mission-export',
				builder: (context, state) => const MissionExportScreen(),
			),
		],
	);
});
