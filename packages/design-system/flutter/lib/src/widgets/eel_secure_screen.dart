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

import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../theme/eel_theme.dart';

/// Blocks screen capture / screenshots: FLAG_SECURE on Android,
/// UIScreen capture observation on iOS. Applied to incident, lookup, kiosk,
/// countersign, diagnostics, tally-verify screens (plan §3.4).
class EelSecureScreen extends StatefulWidget {
  const EelSecureScreen({super.key, required this.child});

  final Widget child;

  @override
  State<EelSecureScreen> createState() => _EelSecureScreenState();
}

class _EelSecureScreenState extends State<EelSecureScreen>
    with WidgetsBindingObserver {
  static const _channel = MethodChannel('eel.design/secure_screen');

  bool _captured = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    _apply();
  }

  Future<void> _apply() async {
    if (kIsWeb) return;
    try {
      if (Platform.isAndroid) {
        await _channel.invokeMethod<void>('setFlagSecure', true);
      } else if (Platform.isIOS) {
        final captured = await _channel
            .invokeMethod<bool>('observeCapture')
            .then((v) => v ?? false);
        if (mounted) setState(() => _captured = captured);
      }
    } on PlatformException {
      // Best-effort; hardening continues even if channel unavailable (tests).
    }
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed && !kIsWeb && !Platform.isAndroid) {
      _apply();
    }
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    if (!kIsWeb) {
      try {
        if (Platform.isAndroid) {
          _channel.invokeMethod<void>('setFlagSecure', false);
        }
      } on PlatformException {
        // ignore
      }
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final c = context.eelColors;
    return Stack(
      children: [
        widget.child,
        if (_captured)
          Positioned.fill(
            child: Semantics(
              label: 'Screen capture detected',
              child: Container(
                color: c.background,
                alignment: Alignment.center,
                padding: const EdgeInsets.all(EelSpacing.s6),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(Icons.no_photography_outlined,
                        size: 48, color: c.danger),
                    const SizedBox(height: EelSpacing.s4),
                    Text(
                      'Screen capture blocked',
                      style: Theme.of(context)
                          .textTheme
                          .titleLarge
                          ?.copyWith(color: c.textPrimary),
                    ),
                    const SizedBox(height: EelSpacing.s2),
                    Text(
                      'This screen contains sensitive data.',
                      style: TextStyle(color: c.textSecondary),
                    ),
                  ],
                ),
              ),
            ),
          ),
      ],
    );
  }
}
