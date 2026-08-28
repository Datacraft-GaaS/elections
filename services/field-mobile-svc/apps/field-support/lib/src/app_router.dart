import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'features/ticket_queue/ticket_queue_screen.dart';
import 'features/diagnostics/diagnostics_screen.dart';
import 'features/warranty/warranty_screen.dart';
import 'features/shift_report/shift_report_screen.dart';

final appRouterProvider = Provider<GoRouter>((ref) {
  return GoRouter(
    initialLocation: '/tickets',
    routes: [
      GoRoute(
        path: '/tickets',
        builder: (context, state) => const TicketQueueScreen(),
      ),
      GoRoute(
        path: '/diagnostics/:kitSerial',
        builder: (context, state) => DiagnosticsScreen(
          kitSerial: state.pathParameters['kitSerial']!,
        ),
      ),
      GoRoute(
        path: '/warranty',
        builder: (context, state) => const WarrantyScreen(),
      ),
      GoRoute(
        path: '/shift-report',
        builder: (context, state) => const ShiftReportScreen(),
      ),
    ],
  );
});
