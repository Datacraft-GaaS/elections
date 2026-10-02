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
