import 'package:eel_kit_sdk/telemetry.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'l10n/app_localizations.dart';
import 'src/app_router.dart';

void main() async {
	WidgetsFlutterBinding.ensureInitialized();
	await TelemetryService.init(
		appId: 'party-agent',
		personaRole: 'party_agent',
	);
	runApp(const ProviderScope(child: EelPartyAgentApp()));
}

class EelPartyAgentApp extends ConsumerWidget {
	const EelPartyAgentApp({super.key});

	@override
	Widget build(BuildContext context, WidgetRef ref) {
		final router = ref.watch(appRouterProvider);
		return MaterialApp.router(
			title: 'Party Agent',
			routerConfig: router,
			localizationsDelegates: const [
				AppLocalizations.delegate,
				GlobalMaterialLocalizations.delegate,
				GlobalWidgetsLocalizations.delegate,
				GlobalCupertinoLocalizations.delegate,
			],
			supportedLocales: AppLocalizations.supportedLocales,
		);
	}
}
