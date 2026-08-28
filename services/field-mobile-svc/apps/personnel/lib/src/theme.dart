import 'package:flutter/material.dart';

/// Design language shared with the poll-worker portal ("The Polling Clerk's
/// Ledger"): ballot-paper ground, ballot-box green ink, one stamp-pad red
/// reserved for the oath/urgent, and dawn gold for money. Serif display,
/// monospace for reference numbers and hashes.
class EelColors {
  EelColors._();

  static const paper = Color(0xFFF1F0E8);
  static const paper2 = Color(0xFFE9E8DE);
  static const raised = Color(0xFFFBFCF5);

  static const ink = Color(0xFF12231A);
  static const inkSoft = Color(0xFF43564A);
  static const inkFaint = Color(0xFF58665D);

  static const green = Color(0xFF1B6A43);
  static const greenDeep = Color(0xFF0E4A2E);
  static const greenTint = Color(0xFFE3EFE6);
  static const greenLine = Color(0xFFC3DCC9);

  static const seal = Color(0xFFB3261E);
  static const sealDeep = Color(0xFF8F1D16);
  static const sealTint = Color(0xFFFBE9E7);

  static const gold = Color(0xFFB4862F);
  static const goldTint = Color(0xFFF6EEDD);

  static const line = Color(0xFFD6D3C3);
  static const lineStrong = Color(0xFFB9B6A6);
}

/// Fonts: a characterful serif for display, the platform default for body,
/// and a monospace for reference numbers and hashes.
class EelFonts {
  EelFonts._();

  static const display = 'Georgia';
  static const displayFallback = <String>['serif'];
  static const mono = 'monospace';
}

class EelTypography {
  EelTypography._();

  static const display = TextStyle(
    fontFamily: EelFonts.display,
    fontFamilyFallback: EelFonts.displayFallback,
    fontWeight: FontWeight.w600,
    letterSpacing: -0.4,
    color: EelColors.ink,
  );

  static const mono = TextStyle(
    fontFamily: EelFonts.mono,
    letterSpacing: 0.02,
  );
}

ThemeData buildEelTheme() {
  final base = ColorScheme.fromSeed(seedColor: EelColors.green);
  final scheme = base.copyWith(
    primary: EelColors.green,
    onPrimary: Colors.white,
    primaryContainer: EelColors.greenTint,
    onPrimaryContainer: EelColors.greenDeep,
    secondary: EelColors.seal,
    onSecondary: Colors.white,
    error: EelColors.seal,
    onError: Colors.white,
    outline: EelColors.lineStrong,
    outlineVariant: EelColors.line,
  );

  final baseText = ThemeData.light().textTheme;

  return ThemeData(
    useMaterial3: true,
    colorScheme: scheme,
    scaffoldBackgroundColor: EelColors.paper,
    canvasColor: EelColors.paper,
    textTheme: baseText.copyWith(
      displayLarge: EelTypography.display.copyWith(fontSize: 34, height: 1.12),
      displayMedium: EelTypography.display.copyWith(fontSize: 28, height: 1.12),
      headlineLarge: EelTypography.display.copyWith(fontSize: 26, height: 1.15),
      headlineMedium: EelTypography.display.copyWith(fontSize: 22, height: 1.2),
      titleLarge: EelTypography.display.copyWith(fontSize: 20, height: 1.2),
      titleMedium: const TextStyle(
        fontWeight: FontWeight.w600,
        fontSize: 16,
        color: EelColors.ink,
      ),
      bodyLarge: const TextStyle(fontSize: 16, height: 1.55, color: EelColors.ink),
      bodyMedium: const TextStyle(fontSize: 14, height: 1.5, color: EelColors.inkSoft),
      bodySmall: const TextStyle(fontSize: 12.5, color: EelColors.inkFaint),
      labelLarge: const TextStyle(fontWeight: FontWeight.w600, fontSize: 14),
    ),
    appBarTheme: const AppBarTheme(
      backgroundColor: EelColors.paper,
      foregroundColor: EelColors.ink,
      surfaceTintColor: Colors.transparent,
      elevation: 0,
      scrolledUnderElevation: 0,
      centerTitle: false,
      titleTextStyle: TextStyle(
        fontFamily: EelFonts.display,
        fontFamilyFallback: EelFonts.displayFallback,
        fontWeight: FontWeight.w600,
        fontSize: 20,
        color: EelColors.ink,
      ),
    ),
    cardTheme: CardThemeData(
      color: EelColors.raised,
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(14),
        side: const BorderSide(color: EelColors.line),
      ),
    ),
    filledButtonTheme: FilledButtonThemeData(
      style: FilledButton.styleFrom(
        backgroundColor: EelColors.green,
        foregroundColor: Colors.white,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(999)),
        padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 14),
        textStyle: const TextStyle(fontWeight: FontWeight.w600, fontSize: 15),
      ),
    ),
    outlinedButtonTheme: OutlinedButtonThemeData(
      style: OutlinedButton.styleFrom(
        foregroundColor: EelColors.ink,
        side: const BorderSide(color: EelColors.lineStrong),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(999)),
        textStyle: const TextStyle(fontWeight: FontWeight.w600),
      ),
    ),
    navigationBarTheme: const NavigationBarThemeData(
      backgroundColor: EelColors.raised,
      indicatorColor: EelColors.greenTint,
      height: 68,
      labelTextStyle: WidgetStatePropertyAll(
        TextStyle(fontSize: 12, fontWeight: FontWeight.w600),
      ),
    ),
    dividerTheme: const DividerThemeData(color: EelColors.line, thickness: 1),
    snackBarTheme: SnackBarThemeData(
      backgroundColor: EelColors.ink,
      contentTextStyle: const TextStyle(color: EelColors.paper),
      behavior: SnackBarBehavior.floating,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
    ),
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: Colors.white,
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(9),
        borderSide: const BorderSide(color: EelColors.line),
      ),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(9),
        borderSide: const BorderSide(color: EelColors.line),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(9),
        borderSide: const BorderSide(color: EelColors.green, width: 2),
      ),
    ),
  );
}
