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
