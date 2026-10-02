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

import 'package:go_router/go_router.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

import 'features/onboarding/onboarding_screen.dart';
import 'features/otp/otp_screen.dart';
import 'features/register/lookup_screen.dart';
import 'features/stations/station_finder_screen.dart';
import 'features/results/results_screen.dart';
import 'features/incident/report_screen.dart';
import 'features/bookmarks/bookmarks_screen.dart';

final appRouter = Provider<GoRouter>((ref) => GoRouter(
  initialLocation: '/onboarding',
  routes: [
    GoRoute(path: '/onboarding', builder: (_, __) => const OnboardingScreen()),
    GoRoute(path: '/otp', builder: (_, __) => const OtpScreen()),
    GoRoute(path: '/register-lookup', builder: (_, __) => const RegisterLookupScreen()),
    GoRoute(path: '/station-finder', builder: (_, __) => const StationFinderScreen()),
    GoRoute(path: '/results', builder: (_, __) => const ResultsScreen()),
    GoRoute(path: '/incident-report', builder: (_, __) => const IncidentReportScreen()),
    GoRoute(path: '/bookmarks', builder: (_, __) => const BookmarksScreen()),
  ],
));
