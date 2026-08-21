import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';

// eel supports 11 languages: English, Kiswahili, Kikuyu, Luo, Kamba,
// Kalenjin, Luhya, Meru, Somali, Mijikenda, Turkana.
const List<Locale> eelSupportedLocales = [
  Locale('en'),
  Locale('sw'),
  Locale('ki'),
  Locale('luo'),
  Locale('kam'),
  Locale('kln'),
  Locale('luy'),
  Locale('mer'),
  Locale('so'),
  Locale('mij'),
  Locale('tuv'),
];

const List<LocalizationsDelegate<dynamic>> eelLocalizationDelegates = [
  GlobalMaterialLocalizations.delegate,
  GlobalWidgetsLocalizations.delegate,
  GlobalCupertinoLocalizations.delegate,
];
