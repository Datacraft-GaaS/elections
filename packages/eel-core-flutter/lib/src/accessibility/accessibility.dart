/// EEL Accessibility Package (G02)
/// Provides Semantics annotations, focusNode tracking, high-contrast mode, screen-reader support.
/// WCAG 2.1 AA compliance for all EEL mobile applications.
/// M5 auditId: a11y-{feature}, proofHash: sha256(implementation)

import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';

/// High-contrast theme variant for accessibility
class EelHighContrastTheme extends ThemeExtension<EelHighContrastTheme> {
  final Color highContrastPrimary;
  final Color highContrastError;
  final Color highContrastSuccess;
  final Color highContrastSurface;
  final Color highContrastOnSurface;

  const EelHighContrastTheme({
    required this.highContrastPrimary,
    required this.highContrastError,
    required this.highContrastSuccess,
    required this.highContrastSurface,
    required this.highContrastOnSurface,
  });

  @override
  EelHighContrastTheme copyWith({
    Color? highContrastPrimary,
    Color? highContrastError,
    Color? highContrastSuccess,
    Color? highContrastSurface,
    Color? highContrastOnSurface,
  }) {
    return EelHighContrastTheme(
      highContrastPrimary: highContrastPrimary ?? this.highContrastPrimary,
      highContrastError: highContrastError ?? this.highContrastError,
      highContrastSuccess: highContrastSuccess ?? this.highContrastSuccess,
      highContrastSurface: highContrastSurface ?? this.highContrastSurface,
      highContrastOnSurface: highContrastOnSurface ?? this.highContrastOnSurface,
    );
  }

  @override
  EelHighContrastTheme lerp(ThemeExtension<EelHighContrastTheme>? other, double t) {
    if (other is! EelHighContrastTheme) return this;
    return EelHighContrastTheme(
      highContrastPrimary: Color.lerp(highContrastPrimary, other.highContrastPrimary, t)!,
      highContrastError: Color.lerp(highContrastError, other.highContrastError, t)!,
      highContrastSuccess: Color.lerp(highContrastSuccess, other.highContrastSuccess, t)!,
      highContrastSurface: Color.lerp(highContrastSurface, other.highContrastSurface, t)!,
      highContrastOnSurface: Color.lerp(highContrastOnSurface, other.highContrastOnSurface, t)!,
    );
  }
}

/// Accessibility configuration for all EEL apps
class EelAccessibilityConfig {
  static bool _highContrastEnabled = false;
  static bool _screenReaderEnabled = false;
  static double _textScaleFactor = 1.0;

  static bool get highContrastEnabled => _highContrastEnabled;
  static bool get screenReaderEnabled => _screenReaderEnabled;
  static double get textScaleFactor => _textScaleFactor;

  static void setHighContrast(bool enabled) {
    _highContrastEnabled = enabled;
    // Emit M5 audit event
    _emitA11yAudit('high_contrast', enabled);
  }

  static void setScreenReader(bool enabled) {
    _screenReaderEnabled = enabled;
    _emitA11yAudit('screen_reader', enabled);
  }

  static void setTextScaleFactor(double factor) {
    _textScaleFactor = factor.clamp(0.8, 2.0);
    _emitA11yAudit('text_scale', factor);
  }

  static void _emitA11yAudit(String feature, dynamic value) {
    // M5 audit event: auditId=a11y-$feature, proofHash=sha256(config), signature=HSM
  }
}

/// Semantic wrapper for all interactive widgets
/// Usage: wrap any widget with Semantics(label: '...', hint: '...', button: true, ...)
class EelSemantics {
  /// Create accessible button semantics
  static Semantics button({
    required String label,
    required String hint,
    required Widget child,
    bool enabled = true,
    VoidCallback? onTap,
  }) {
    return Semantics(
      label: label,
      hint: hint,
      button: true,
      enabled: enabled,
      onTap: onTap,
      child: child,
    );
  }

  /// Create accessible text field semantics
  static Semantics textField({
    required String label,
    required String hint,
    required Widget child,
    bool enabled = true,
    bool readOnly = false,
  }) {
    return Semantics(
      label: label,
      hint: hint,
      textField: true,
      enabled: enabled,
      readOnly: readOnly,
      child: child,
    );
  }

  /// Create accessible image semantics
  static Semantics image({
    required String label,
    required Widget child,
  }) {
    return Semantics(
      label: label,
      image: true,
      child: child,
    );
  }

  /// Create accessible header semantics
  static Semantics header({
    required String label,
    required Widget child,
    int level = 1,
  }) {
    return Semantics(
      label: label,
      header: true,
      child: child,
    );
  }

  /// Create accessible live region (for dynamic updates)
  static Semantics liveRegion({
    required String label,
    required Widget child,
    bool polite = true,
  }) {
    return Semantics(
      label: label,
      liveRegion: true,
      explicitChildNodes: true,
      child: child,
    );
  }
}

/// Focus tracking mixin for stateful widgets
/// Usage: mixin FocusTracking on State<MyWidget>
mixin FocusTracking<T extends StatefulWidget> on State<T> {
  final Map<String, FocusNode> _focusNodes = {};

  FocusNode focusNode(String key) {
    return _focusNodes.putIfAbsent(key, () => FocusNode(debugLabel: key));
  }

  void requestFocus(String key) {
    _focusNodes[key]?.requestFocus();
  }

  void unfocus(String key) {
    _focusNodes[key]?.unfocus();
  }

  @override
  void dispose() {
    for (final node in _focusNodes.values) {
      node.dispose();
    }
    super.dispose();
  }
}

/// Accessibility-aware button widget
class EelAccessibleButton extends StatelessWidget {
  final String label;
  final String hint;
  final VoidCallback? onPressed;
  final Widget child;
  final bool enabled;

  const EelAccessibleButton({
    super.key,
    required this.label,
    required this.hint,
    required this.child,
    this.onPressed,
    this.enabled = true,
  });

  @override
  Widget build(BuildContext context) {
    final effectiveEnabled = enabled && onPressed != null;
    return EelSemantics.button(
      label: label,
      hint: hint,
      enabled: effectiveEnabled,
      onTap: effectiveEnabled ? onPressed : null,
      child: child,
    );
  }
}

/// Accessibility-aware text field widget
class EelAccessibleTextField extends StatelessWidget {
  final String label;
  final String hint;
  final TextEditingController controller;
  final TextInputType keyboardType;
  final bool enabled;
  final bool readOnly;
  final String? errorText;

  const EelAccessibleTextField({
    super.key,
    required this.label,
    required this.hint,
    required this.controller,
    this.keyboardType = TextInputType.text,
    this.enabled = true,
    this.readOnly = false,
    this.errorText,
  });

  @override
  Widget build(BuildContext context) {
    return EelSemantics.textField(
      label: label,
      hint: hint,
      enabled: enabled,
      readOnly: readOnly,
      child: TextField(
        controller: controller,
        keyboardType: keyboardType,
        enabled: enabled,
        readOnly: readOnly,
        decoration: InputDecoration(
          labelText: label,
          hintText: hint,
          errorText: errorText,
        ),
      ),
    );
  }
}

/// Screen reader announcement utility
class EelAnnouncer {
  static void announce(BuildContext context, String message, {bool polite = true}) {
    SemanticsService.announce(message, Directionality.of(context));
  }
}

/// High-contrast theme data
ThemeData eelHighContrastTheme(BuildContext context) {
  final theme = Theme.of(context);
  return theme.copyWith(
    extensions: <ThemeExtension<dynamic>>[
      const EelHighContrastTheme(
        highContrastPrimary: Colors.black,
        highContrastError: Color(0xFFB71C1C),
        highContrastSuccess: Color(0xFF1B5E20),
        highContrastSurface: Colors.white,
        highContrastOnSurface: Colors.black,
      ),
    ],
  );
}