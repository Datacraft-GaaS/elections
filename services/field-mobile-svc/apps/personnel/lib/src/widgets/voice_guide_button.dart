import 'package:flutter/material.dart';

/// Voice-guided workflow button for low-literacy users.
class VoiceGuideButton extends StatelessWidget {
  final String label;
  final String voiceText;
  final VoidCallback onPressed;

  const VoiceGuideButton({
    super.key,
    required this.label,
    required this.voiceText,
    required this.onPressed,
  });

  @override
  Widget build(BuildContext context) {
    return ElevatedButton.icon(
      onPressed: onPressed,
      icon: const Icon(Icons.volume_up),
      label: Text(label),
    );
  }
}
