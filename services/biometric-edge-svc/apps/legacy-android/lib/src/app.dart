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
import 'package:flutter_localizations/flutter_localizations.dart';

import 'legacy_home.dart';

const bool _legacyShimEnabled = true; // controlled by compile-time feature flag

class LegacyAndroidApp extends StatelessWidget {
  const LegacyAndroidApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'eel Legacy Kit',
      localizationsDelegates: const [
        GlobalMaterialLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
      ],
      supportedLocales: const [
        Locale('en'), Locale('sw'), Locale('ki'), Locale('luo'), Locale('kam'),
        Locale('kln'), Locale('luy'), Locale('mer'), Locale('so'), Locale('mij'),
        Locale('tuv'),
      ],
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: const Color(0xFF990000)),
        useMaterial3: true,
      ),
      home: const LegacyHomeScreen(shimEnabled: _legacyShimEnabled),
    );
  }
}
