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
import 'package:flutter_localizations/flutter_localizations.dart';

import 'identification_workflow.dart';

const List<Locale> _supportedLocales = [
  Locale('en'), Locale('sw'), Locale('ki'), Locale('luo'), Locale('kam'),
  Locale('kln'), Locale('luy'), Locale('mer'), Locale('so'), Locale('mij'),
  Locale('tuv'),
];

class BviKitApp extends StatelessWidget {
  const BviKitApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'eel BVI Kit',
      navigatorKey: eelNavigatorKey,
      localizationsDelegates: const [
        GlobalMaterialLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
      ],
      supportedLocales: _supportedLocales,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: const Color(0xFF003399)),
        useMaterial3: true,
      ),
      home: const IdentificationWorkflowScreen(),
    );
  }
}
