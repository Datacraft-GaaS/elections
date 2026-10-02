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
				builder: (context, state) => const ObserverLoginScreen(),
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
