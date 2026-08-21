import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

import 'features/scan/scan_result.dart';
import 'features/scan/scan_result_screen.dart';
import 'features/scan/scan_screen.dart';

final _router = GoRouter(
  initialLocation: '/scan',
  routes: [
    GoRoute(
      path: '/scan',
      builder: (context, state) => const ScanScreen(),
    ),
    GoRoute(
      path: '/scan/result',
      builder: (context, state) {
        final result = state.extra as ScanResult;
        return ScanResultScreen(result: result);
      },
    ),
  ],
);

class EelAccredScannerApp extends HookConsumerWidget {
  const EelAccredScannerApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return MaterialApp.router(
      title: 'eel Accreditation Scanner',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: const Color(0xFF006600)),
        useMaterial3: true,
      ),
      routerConfig: _router,
    );
  }
}
