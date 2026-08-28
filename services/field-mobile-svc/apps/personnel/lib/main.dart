import 'package:eel_kit_sdk/telemetry.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'src/app_router.dart';
import 'src/theme.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await TelemetryService.init(
    appId: 'personnel',
    personaRole: 'personnel_officer',
  );
  runApp(const ProviderScope(child: EelPersonnelApp()));
}

class EelPersonnelApp extends ConsumerWidget {
  const EelPersonnelApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final router = ref.watch(appRouterProvider);
    return MaterialApp.router(
      title: 'EEL Personnel',
      theme: buildEelTheme(),
      routerConfig: router,
    );
  }
}
