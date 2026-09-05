import 'package:flutter/material.dart';

import 'eel_colors.dart';

export 'eel_colors.dart';

/// Builds light and dark [ThemeData] from tokens.json (via [EelColors]).
///
/// The theme exposes [EelColors] as a [ThemeExtension] so components can
/// reach semantic tokens directly: `context.eelColors.success`.
abstract final class EelTheme {
  static const fontFamily = EelTypography.fontFamily;

  static ThemeData light() => _build(Brightness.light);
  static ThemeData dark() => _build(Brightness.dark);

  static ThemeData _build(Brightness brightness) {
    final c = eelColorsOf(brightness);
    final scheme = ColorScheme(
      brightness: brightness,
      primary: c.brandPrimary,
      onPrimary: c.textInverse,
      secondary: c.brandSecondary,
      onSecondary: c.textInverse,
      error: c.danger,
      onError: c.textInverse,
      surface: c.background,
      onSurface: c.textPrimary,
      surfaceContainerHighest: c.surface,
      onSurfaceVariant: c.textSecondary,
      outline: c.border,
      outlineVariant: c.border,
    );

    final base = ThemeData(
      useMaterial3: true,
      brightness: brightness,
      colorScheme: scheme,
      fontFamily: EelTypography.fontFamily,
    );

    return base.copyWith(
      extensions: [c],
      scaffoldBackgroundColor: c.background,
      splashFactory: InkSparkle.splashFactory,
      textTheme: _textTheme(base.textTheme, c),
      appBarTheme: AppBarTheme(
        backgroundColor: c.background,
        foregroundColor: c.textPrimary,
        elevation: 0,
        centerTitle: false,
        titleTextStyle: base.textTheme.titleLarge?.copyWith(
          color: c.textPrimary,
          fontWeight: FontWeight.w600,
        ),
      ),
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          backgroundColor: c.brandPrimary,
          foregroundColor: c.textInverse,
          minimumSize: const Size(0, EelSizing.enforcedTouch),
          padding: const EdgeInsets.symmetric(
            horizontal: EelSpacing.s4,
            vertical: EelSpacing.s3,
          ),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(EelRadius.lg),
          ),
          textStyle: const TextStyle(
            fontSize: EelTypography.base,
            fontWeight: FontWeight.w600,
          ),
        ),
      ),
      filledButtonTheme: FilledButtonThemeData(
        style: FilledButton.styleFrom(
          minimumSize: const Size(0, EelSizing.enforcedTouch),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(EelRadius.lg),
          ),
        ),
      ),
      textButtonTheme: TextButtonThemeData(
        style: TextButton.styleFrom(
          foregroundColor: c.textLink,
          minimumSize: const Size(0, EelSizing.enforcedTouch),
        ),
      ),
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: OutlinedButton.styleFrom(
          foregroundColor: c.textPrimary,
          side: BorderSide(color: c.border),
          minimumSize: const Size(0, EelSizing.enforcedTouch),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(EelRadius.lg),
          ),
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: c.surface,
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(EelRadius.lg),
          borderSide: BorderSide(color: c.border),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(EelRadius.lg),
          borderSide: BorderSide(color: c.border),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(EelRadius.lg),
          borderSide: const BorderSide(
            color: EelFocus.ringColor,
            width: 2,
          ),
        ),
        errorBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(EelRadius.lg),
          borderSide: BorderSide(color: c.danger),
        ),
        focusedErrorBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(EelRadius.lg),
          borderSide: BorderSide(color: c.danger, width: 2),
        ),
        contentPadding: const EdgeInsets.symmetric(
          horizontal: EelSpacing.s3,
          vertical: EelSpacing.s3,
        ),
      ),
      dividerTheme: DividerThemeData(color: c.border, thickness: 1),
      chipTheme: ChipThemeData(
        backgroundColor: c.surface,
        side: BorderSide(color: c.border),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(EelRadius.full),
        ),
        labelStyle: TextStyle(color: c.textPrimary, fontSize: EelTypography.sm),
      ),
      snackBarTheme: SnackBarThemeData(
        backgroundColor: c.textPrimary,
        contentTextStyle: TextStyle(color: c.textInverse),
        behavior: SnackBarBehavior.floating,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(EelRadius.lg),
        ),
      ),
    );
  }

  static TextTheme _textTheme(TextTheme base, EelColors c) {
    return base
        .copyWith(
          displayLarge:
              base.displayLarge?.copyWith(fontSize: EelTypography.xl4),
          displayMedium:
              base.displayMedium?.copyWith(fontSize: EelTypography.xl3),
          displaySmall:
              base.displaySmall?.copyWith(fontSize: EelTypography.xl2),
          headlineMedium:
              base.headlineMedium?.copyWith(fontSize: EelTypography.xl),
          headlineSmall:
              base.headlineSmall?.copyWith(fontSize: EelTypography.lg),
          titleLarge: base.titleLarge?.copyWith(fontSize: EelTypography.lg),
          bodyLarge: base.bodyLarge?.copyWith(
            fontSize: EelTypography.base,
            height: 1.5,
          ),
          bodyMedium: base.bodyMedium?.copyWith(
            fontSize: EelTypography.sm,
            height: 1.5,
          ),
          bodySmall: base.bodySmall?.copyWith(fontSize: EelTypography.xs),
          labelLarge: base.labelLarge?.copyWith(
            fontSize: EelTypography.base,
            fontWeight: FontWeight.w600,
          ),
          labelMedium: base.labelMedium?.copyWith(fontSize: EelTypography.sm),
          labelSmall: base.labelSmall?.copyWith(fontSize: EelTypography.xs),
        )
        .apply(bodyColor: c.textPrimary, displayColor: c.textPrimary);
  }
}

/// Accessor for [EelColors] off any [BuildContext].
extension EelColorsX on BuildContext {
  EelColors get eelColors =>
      Theme.of(this).extension<EelColors>() ??
      eelColorsOf(Theme.of(this).brightness);
}
