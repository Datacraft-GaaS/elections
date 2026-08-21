import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'l10n/app_localizations.dart';
import 'src/app_router.dart';

void main() {
	runApp(const ProviderScope(child: EelObserverApp()));
}

class EelObserverApp extends ConsumerWidget {
	const EelObserverApp({super.key});

	@override
	Widget build(BuildContext context, WidgetRef ref) {
		final router = ref.watch(appRouterProvider);
		return MaterialApp.router(
			title: 'Observer',
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
