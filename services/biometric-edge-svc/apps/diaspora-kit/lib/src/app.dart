import 'package:flutter/material.dart';

import 'station_screen.dart';

class DiasporaKitApp extends StatelessWidget {
  const DiasporaKitApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'eel Diaspora Kit',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: const Color(0xFF006847)),
        useMaterial3: true,
      ),
      home: const StationScreen(),
    );
  }
}
