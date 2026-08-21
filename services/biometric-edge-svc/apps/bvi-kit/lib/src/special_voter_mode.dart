/// Special voter mode additions to bvi-kit (WP-5.5, M25).
///
/// Minimal kit-side mode selector and banner — the full special-kit UI lives in
/// services/support-admin-svc/apps/special-kit/. This file exposes the mode
/// enum and the colour-coded banner widget so bvi-kit can switch into
/// special-voter state without a separate app binary.

library;

import 'package:flutter/material.dart';

/// Maps directly to SpecialVoterCategory in the Rust domain crate.
enum SpecialVoterMode {
  standard,
  remandPrison,
  hospital,
  barracks,
  refugeeCamp,
  idpCamp,
}

extension SpecialVoterModeX on SpecialVoterMode {
  String get label {
    switch (this) {
      case SpecialVoterMode.standard:
        return 'Standard';
      case SpecialVoterMode.remandPrison:
        return 'Remand Prison';
      case SpecialVoterMode.hospital:
        return 'Hospital';
      case SpecialVoterMode.barracks:
        return 'KDF / GSU / Prison Service';
      case SpecialVoterMode.refugeeCamp:
        return 'Refugee Camp';
      case SpecialVoterMode.idpCamp:
        return 'IDP Camp';
    }
  }

  /// Colour-coded banner per FR-M25-048.
  Color get bannerColor {
    switch (this) {
      case SpecialVoterMode.standard:
        return const Color(0xFF003399);
      case SpecialVoterMode.remandPrison:
        return const Color(0xFFFF6600);
      case SpecialVoterMode.hospital:
        return const Color(0xFF0055AA);
      case SpecialVoterMode.barracks:
        return const Color(0xFF007733);
      case SpecialVoterMode.refugeeCamp:
      case SpecialVoterMode.idpCamp:
        return const Color(0xFF884499);
    }
  }

  /// Requires dual biometric open per FR-M25-002 / FR-M25-017.
  bool get requiresDualBiometricOpen =>
      this == SpecialVoterMode.remandPrison || this == SpecialVoterMode.barracks;

  String get formVariant {
    switch (this) {
      case SpecialVoterMode.remandPrison:
        return '34A-R';
      case SpecialVoterMode.hospital:
        return '34A-H';
      case SpecialVoterMode.barracks:
        return '34A-B';
      case SpecialVoterMode.refugeeCamp:
      case SpecialVoterMode.idpCamp:
        return '34A-I';
      case SpecialVoterMode.standard:
        return '34A';
    }
  }
}

/// Compact banner shown at the top of all special-mode screens (FR-M25-048).
class SpecialModeBanner extends StatelessWidget {
  const SpecialModeBanner({super.key, required this.mode});

  final SpecialVoterMode mode;

  @override
  Widget build(BuildContext context) {
    if (mode == SpecialVoterMode.standard) return const SizedBox.shrink();
    return Container(
      width: double.infinity,
      color: mode.bannerColor,
      padding: const EdgeInsets.symmetric(vertical: 6, horizontal: 16),
      child: Text(
        'MODE: ${mode.label.toUpperCase()}  |  FORM ${mode.formVariant}',
        style: const TextStyle(
          color: Colors.white,
          fontWeight: FontWeight.bold,
          fontSize: 13,
          letterSpacing: 0.5,
        ),
        textAlign: TextAlign.center,
        semanticsLabel: 'Special voter mode: ${mode.label}',
      ),
    );
  }
}

/// State holder for the active special voter mode in the kit session.
/// Injected via Riverpod / InheritedWidget at app root.
class SpecialVoterModeNotifier extends ChangeNotifier {
  SpecialVoterMode _mode = SpecialVoterMode.standard;
  bool _refugeeVotingEnabled = false;

  SpecialVoterMode get mode => _mode;
  bool get refugeeVotingEnabled => _refugeeVotingEnabled;

  /// Switch mode; caller must have verified provisioning config from server.
  void activateMode(SpecialVoterMode m, {bool refugeeFlag = false}) {
    if (m == SpecialVoterMode.refugeeCamp && !refugeeFlag) {
      throw StateError(
        'FR-M25-022: REFUGEE_VOTING_ENABLED flag is false; cannot activate refugee mode',
      );
    }
    _mode = m;
    _refugeeVotingEnabled = refugeeFlag;
    notifyListeners();
  }

  void reset() {
    _mode = SpecialVoterMode.standard;
    _refugeeVotingEnabled = false;
    notifyListeners();
  }
}
