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
