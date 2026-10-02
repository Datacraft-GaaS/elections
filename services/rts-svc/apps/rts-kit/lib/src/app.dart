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

import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import 'rts_workflow/scan.dart';
import 'rts_workflow/ocr_preview.dart';
import 'rts_workflow/sign.dart';
import 'rts_workflow/transmit.dart';

final _router = GoRouter(
  navigatorKey: eelNavigatorKey,
  initialLocation: '/scan',
  routes: [
    GoRoute(
      path: '/scan',
      builder: (ctx, state) => const ScanScreen(),
    ),
    GoRoute(
      path: '/ocr-preview',
      builder: (ctx, state) {
        final args = state.extra! as OcrPreviewArgs;
        return OcrPreviewScreen(args: args);
      },
    ),
    GoRoute(
      path: '/sign',
      builder: (ctx, state) {
        final args = state.extra! as SignArgs;
        return SignScreen(args: args);
      },
    ),
    GoRoute(
      path: '/transmit',
      builder: (ctx, state) {
        final args = state.extra! as TransmitArgs;
        return TransmitScreen(args: args);
      },
    ),
  ],
);

class EelRtsKitApp extends ConsumerWidget {
  const EelRtsKitApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return MaterialApp.router(
      title: 'eel RTS Kit',
      routerConfig: _router,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: const Color(0xFF006B3F)),
        useMaterial3: true,
      ),
      darkTheme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF006B3F),
          brightness: Brightness.dark,
        ),
        useMaterial3: true,
      ),
    );
  }
}
