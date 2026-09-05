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
