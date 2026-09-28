import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'l10n/app_localizations.dart';
import 'src/app_router.dart';

void main() async {
	WidgetsFlutterBinding.ensureInitialized();
	await TelemetryService.init(
		appId: 'poll-clerk',
		personaRole: 'poll_clerk',
	);
	runApp(const ProviderScope(child: EelPollClerkApp()));
}

class EelPollClerkApp extends ConsumerWidget {
	const EelPollClerkApp({super.key});

	@override
	Widget build(BuildContext context, WidgetRef ref) {
		final router = ref.watch(appRouterProvider);
		return MaterialApp.router(
			title: 'Poll Clerk',
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
