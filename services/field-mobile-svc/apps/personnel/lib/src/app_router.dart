import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'app_shell.dart';
import 'features/assignment/assignment_screen.dart';
import 'features/check_in/check_in_screen.dart';
import 'features/status_ping/status_ping_screen.dart';
import 'features/check_out/check_out_screen.dart';
import 'features/incident/incident_screen.dart';
import 'workforce/duty_screen.dart';
import 'workforce/notifications_screen.dart';

final appRouterProvider = Provider<GoRouter>((ref) {
  return GoRouter(
    initialLocation: '/duty',
    routes: [
      StatefulShellRoute.indexedStack(
        builder: (context, state, navigationShell) =>
            AppShell(navigationShell: navigationShell),
        branches: [
          StatefulShellBranch(
            routes: [
              GoRoute(
                path: '/duty',
                builder: (context, state) => const DutyScreen(),
              ),
            ],
          ),
          StatefulShellBranch(
            routes: [
              GoRoute(
                path: '/notifications',
                builder: (context, state) => const NotificationsScreen(),
              ),
            ],
          ),
          StatefulShellBranch(
            routes: [
              GoRoute(
                path: '/check-in',
                builder: (context, state) => const CheckInScreen(),
              ),
            ],
          ),
          StatefulShellBranch(
            routes: [
              GoRoute(
                path: '/status',
                builder: (context, state) => const StatusPingScreen(),
              ),
            ],
          ),
        ],
      ),
      GoRoute(
        path: '/assignment',
        builder: (context, state) => const AssignmentScreen(),
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
