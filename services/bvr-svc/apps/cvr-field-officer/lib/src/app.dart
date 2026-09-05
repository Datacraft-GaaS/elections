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
