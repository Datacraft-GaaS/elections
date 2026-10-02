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
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:eel_design/eel_design.dart' show EelTheme;

import 'screens/home.dart';

class CvrFieldOfficerApp extends ConsumerWidget {
  const CvrFieldOfficerApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return MaterialApp(
      title: 'CVR Field Officer',
      debugShowCheckedModeBanner: false,
      theme: EelTheme.light(),
      darkTheme: EelTheme.dark(),
      localizationsDelegates: const [
        GlobalMaterialLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
      ],
      supportedLocales: const [
        Locale('en'),
        Locale('sw'),
        Locale('ki'),
        Locale('luo'),
        Locale('kal'),
        Locale('kam'),
        Locale('guj'),
        Locale('mer'),
        Locale('mas'),
        Locale('som'),
      ],
      home: const HomeScreen(),
    );
  }
}
