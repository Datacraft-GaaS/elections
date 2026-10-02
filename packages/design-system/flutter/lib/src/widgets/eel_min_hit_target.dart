// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import 'package:flutter/material.dart';

/// Re-export of kit-sdk's hit-target wrapper so apps import a single package.
/// Local copy so eel_design stays dependency-free; kit-sdk re-exports this.
/// Enforces ≥ [minSize] (48 dp default) per WCAG 2.5.5 / plan §5.1.
class EelMinHitTarget extends StatelessWidget {
  const EelMinHitTarget({
    super.key,
    required this.child,
    this.onTap,
    this.semanticsLabel,
    this.minSize = 48.0,
  });

  final Widget child;
  final VoidCallback? onTap;
  final String? semanticsLabel;
  final double minSize;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      label: semanticsLabel,
      button: onTap != null,
      child: GestureDetector(
        onTap: onTap,
        child: ConstrainedBox(
          constraints: BoxConstraints(minWidth: minSize, minHeight: minSize),
          child: Center(child: child),
        ),
      ),
    );
  }
}
