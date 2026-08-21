import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

import 'features/assessment/assessment_screen.dart';
import 'features/modules/modules_screen.dart';
import 'features/sync/sync_service.dart';

final _router = GoRouter(
  initialLocation: '/modules',
  routes: [
    GoRoute(
      path: '/modules',
      builder: (context, state) => const ModulesScreen(),
    ),
    GoRoute(
      path: '/modules/:id',
      builder: (context, state) {
        final id = state.pathParameters['id']!;
        return ModulesScreen(selectedId: id);
      },
    ),
    GoRoute(
      path: '/assessment/:moduleId',
      builder: (context, state) {
        final moduleId = state.pathParameters['moduleId']!;
        return AssessmentScreen(moduleId: moduleId);
      },
    ),
    GoRoute(
      path: '/sync',
      builder: (context, state) => const SyncStatusScreen(),
    ),
  ],
);

class EelLmsOfflineApp extends HookConsumerWidget {
  const EelLmsOfflineApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return MaterialApp.router(
      title: 'eel LMS Offline',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: const Color(0xFF006600)),
        useMaterial3: true,
      ),
      routerConfig: _router,
    );
  }
}
