import 'package:flutter/material.dart';
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
