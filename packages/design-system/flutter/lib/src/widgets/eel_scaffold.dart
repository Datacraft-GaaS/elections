import 'package:flutter/material.dart';

import '../theme/eel_theme.dart';
import '../theme/eel_type.dart';

/// Scaffold with safe-area, banner slot, and bottom action bar slot.
///
/// Field-first ergonomics (plan §5.2): banner appears directly below the
/// app bar; the bottom action bar keeps primary actions in the thumb arc.
class EelScaffold extends StatelessWidget {
  const EelScaffold({
    super.key,
    this.title,
    this.appBar,
    required this.body,
    this.banners = const [],
    this.bottomActionBar,
    this.floatingActionButton,
    this.variant = EelScaffoldVariant.scrollable,
    this.scrollable,
    this.padding = const EdgeInsets.all(EelSpacing.s4),
  });

  /// Title for the app bar. Ignored when [appBar] is supplied.
  final String? title;
  final PreferredSizeWidget? appBar;
  final Widget body;
  final List<Widget> banners;
  final Widget? bottomActionBar;
  final FloatingActionButton? floatingActionButton;

  /// Whether the body scrolls. [variant] sets this; the explicit [scrollable]
  /// argument overrides it for the rare screen that needs to.
  final EelScaffoldVariant variant;
  final bool? scrollable;
  final EdgeInsetsGeometry padding;

  @override
  Widget build(BuildContext context) {
    final colors = context.eelColors;
    final scrolls = scrollable ?? (variant == EelScaffoldVariant.scrollable);
    Widget content = scrolls
        ? SingleChildScrollView(
            child: Padding(padding: padding, child: body),
          )
        : Padding(padding: padding, child: body);

    if (banners.isNotEmpty) {
      content = Column(
        children: [
          ...banners,
          Expanded(child: content),
        ],
      );
    }

    return Scaffold(
      appBar: appBar ??
          (title == null
              ? null
              : AppBar(title: Text(title!, style: EelTextStyle.titleLarge))),
      backgroundColor: colors.background,
      floatingActionButton: floatingActionButton,
      bottomNavigationBar: bottomActionBar == null
          ? null
          : BottomAppBar(
              color: colors.surface,
              elevation: 0,
              child: bottomActionBar!,
            ),
      body: SafeArea(child: content),
    );
  }
}
