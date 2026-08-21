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
