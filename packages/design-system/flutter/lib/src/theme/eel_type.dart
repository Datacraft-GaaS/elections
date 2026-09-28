// SPDX-License-Identifier: Apache-2.0
//
// Text-style and scaffold-variant vocabulary.
//
// The app tier composes screens from a fixed set of type roles and scaffold
// shapes. Naming those roles is what stops each app inventing its own
// fontSize, and it is why the roles live here rather than at every call site.
//
// Values are deliberately platform-independent rather than derived from
// Material's text theme: a field kit renders the same measurements in the
// hand, where ambient theming would make a registration screen inconsistent
// between stations.

import 'package:flutter/material.dart';

/// Type roles used across the app tier.
class EelTextStyle {
  const EelTextStyle._();

  /// Screen-level heading. One per screen.
  static const TextStyle displayMedium = TextStyle(
    fontSize: 28,
    fontWeight: FontWeight.bold,
    height: 1.2,
  );

  /// Section heading within a screen.
  static const TextStyle titleLarge = TextStyle(
    fontSize: 22,
    fontWeight: FontWeight.bold,
    height: 1.25,
  );

  /// Sub-section heading, or a field group label.
  static const TextStyle titleMedium = TextStyle(
    fontSize: 18,
    fontWeight: FontWeight.bold,
    height: 1.3,
  );

  /// Minor heading, or a card title.
  static const TextStyle titleSmall = TextStyle(
    fontSize: 16,
    fontWeight: FontWeight.w600,
    height: 1.3,
  );

  /// Default body text. Most content.
  static const TextStyle bodyMedium = TextStyle(
    fontSize: 14,
    height: 1.4,
  );

  /// Supporting text: help text, timestamps, metadata.
  static const TextStyle bodySmall = TextStyle(
    fontSize: 12,
    height: 1.35,
  );
}

/// How a scaffold arranges its body.
enum EelScaffoldVariant {
  /// The body scrolls. The default for any screen longer than the viewport.
  scrollable,

  /// The body is pinned and does not scroll, with a fixed action bar. Used
  /// where a primary action must stay reachable without scrolling, such as
  /// a capture step.
  fixedAppBar,
}
