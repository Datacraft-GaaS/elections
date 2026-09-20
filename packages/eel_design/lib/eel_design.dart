library eel_design;

import 'package:flutter/material.dart';

/// A scaffold that provides a basic layout with an app bar and body.
class EelScaffold extends StatelessWidget {
  const EelScaffold({
    Key? key,
    required this.title,
    required this.variant,
    required this.padding,
    required this.body,
  }) : super(key: key);

  final String title;
  final EelScaffoldVariant variant;
  final EdgeInsets padding;
  final Widget body;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(title),
      ),
      body: Padding(
        padding: padding,
        child: body,
      ),
    );
  }
}

/// Variants for the EelScaffold.
enum EelScaffoldVariant {
  scrollable,
  fixed,
}

/// Text styles for the Eel design.
class EelTextStyle {
  static TextStyle get titleLarge => const TextStyle(
        fontSize: 22,
        fontWeight: FontWeight.bold,
      );
  static TextStyle get titleMedium => const TextStyle(
        fontSize: 18,
        fontWeight: FontWeight.bold,
      );
  static TextStyle get bodyMedium => const TextStyle(
        fontSize: 14,
      );
}

/// Colors for the Eel design.
class EelColors {
  static Color primary(BuildContext context) =>
      Theme.of(context).colorScheme.primary;

  static Color onPrimary(BuildContext context) =>
      Theme.of(context).colorScheme.onPrimary;
}

/// A card that displays a result with an icon, title, and subtitle.
class EelResultCard extends StatelessWidget {
  const EelResultCard({
    Key? key,
    required this.leading,
    required this.title,
    required this.subtitle,
  }) : super(key: key);

  final Widget leading;
  final Widget title;
  final Widget subtitle;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Row(
          children: [
            leading,
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  title,
                  const SizedBox(height: 4),
                  subtitle,
                ],
              ),
            ),
          ],
        ),
      );
    }
  }

  /// A button that displays a label and an icon.
  class EelButton extends StatelessWidget {
    const EelButton({
      Key? key,
      required this.label,
      required this.icon,
      required this.fullWidth,
      required this.onPressed,
    }) : super(key: key);

    final String label;
    final IconData icon;
    final bool fullWidth;
    final VoidCallback onPressed;

    @override
    Widget build(BuildContext context) {
      final button = ElevatedButton.icon(
        icon: Icon(icon),
        label: Text(label),
        onPressed: onPressed,
      );
      return fullWidth
          ? SizedBox(width: double.infinity, child: button)
          : button;
    }
  }
}
