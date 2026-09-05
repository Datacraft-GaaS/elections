import 'package:flutter/material.dart';

import '../theme/eel_theme.dart';

/// Scaffold with safe-area, banner slot, and bottom action bar slot.
///
/// Field-first ergonomics (plan §5.2): banner appears directly below the
/// app bar; the bottom action bar keeps primary actions in the thumb arc.
class EelScaffold extends StatelessWidget {
  const EelScaffold({
    super.key,
    this.appBar,
    required this.body,
    this.banners = const [],
    this.bottomActionBar,
    this.floatingActionButton,
    this.scrollable = true,
    this.padding = const EdgeInsets.all(EelSpacing.s4),
  });

  final PreferredSizeWidget? appBar;
  final Widget body;
  final List<Widget> banners;
  final Widget? bottomActionBar;
  final FloatingActionButton? floatingActionButton;
  final bool scrollable;
  final EdgeInsetsGeometry padding;

  @override
  Widget build(BuildContext context) {
    final colors = context.eelColors;
    Widget content = scrollable
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
      appBar: appBar,
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
