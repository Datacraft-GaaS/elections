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
