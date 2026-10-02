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

import '../theme/eel_theme.dart';

/// Voice guide toggle button. Speaks the current screen's guidance via OS TTS
/// through the provided callback; announces state via Semantics.
class EelVoiceGuideButton extends StatefulWidget {
  const EelVoiceGuideButton({
    super.key,
    required this.onToggle,
    required this.activeLabel,
    required this.inactiveLabel,
    this.initiallyActive = false,
  });

  /// Called with the new active state; the app wires OS text-to-speech.
  final ValueChanged<bool> onToggle;
  final String activeLabel;
  final String inactiveLabel;
  final bool initiallyActive;

  @override
  State<EelVoiceGuideButton> createState() => _EelVoiceGuideButtonState();
}

class _EelVoiceGuideButtonState extends State<EelVoiceGuideButton> {
  late bool _active = widget.initiallyActive;

  void _toggle() {
    setState(() => _active = !_active);
    widget.onToggle(_active);
  }

  @override
  Widget build(BuildContext context) {
    final c = context.eelColors;
    return Semantics(
      label: _active ? widget.activeLabel : widget.inactiveLabel,
      button: true,
      child: IconButton.filledTonal(
        onPressed: _toggle,
        tooltip: _active ? widget.activeLabel : widget.inactiveLabel,
        icon: Icon(
          _active ? Icons.record_voice_over : Icons.voice_over_off,
          size: 24,
          color: _active ? c.brandPrimary : c.textSecondary,
        ),
        style: IconButton.styleFrom(
          minimumSize:
              const Size(EelSizing.enforcedTouch, EelSizing.enforcedTouch),
        ),
      ),
    );
  }
}
