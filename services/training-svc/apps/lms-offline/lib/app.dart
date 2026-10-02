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
