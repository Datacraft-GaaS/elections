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
