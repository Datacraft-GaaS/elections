import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'features/assignment/assignment_screen.dart';
import 'features/check_in/check_in_screen.dart';
import 'features/status_ping/status_ping_screen.dart';
import 'features/check_out/check_out_screen.dart';
import 'features/incident/incident_screen.dart';

final appRouterProvider = Provider<GoRouter>((ref) {
  return GoRouter(
    initialLocation: '/assignment',
    routes: [
      GoRoute(
        path: '/assignment',
        builder: (context, state) => const AssignmentScreen(),
      ),
      GoRoute(
        path: '/check-in',
        builder: (context, state) => const CheckInScreen(),
      ),
      GoRoute(
        path: '/status',
        builder: (context, state) => const StatusPingScreen(),
      ),
      GoRoute(
        path: '/check-out',
        builder: (context, state) => const CheckOutScreen(),
      ),
      GoRoute(
        path: '/incident',
        builder: (context, state) => const IncidentScreen(),
      ),
    ],
  );
});
