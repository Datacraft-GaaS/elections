/// The OTP-issued bearer token for the anonymous citizen session.
///
/// `RegisterService.lookup` and `fileObjection` both take this as a bearer
/// credential, and the OTP screen receives one from `OtpService.sendOtp`. It
/// used to be discarded at the point of issue, so the two screens that need it
/// could not be wired: they had a button and no way to authorise the call.
///
/// Held in memory only, for the life of the process. It is deliberately not
/// persisted: a bearer token that survives a restart is a token an attacker can
/// use after the citizen has walked away from the device.
library;

import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'register_service.dart';

/// The token issued by the most recent successful OTP request, or null.
///
/// Cleared when the citizen leaves, and on logout, so it cannot outlive the
/// session it was issued for.
final otpTokenProvider = StateProvider<String?>((ref) => null);

/// The register service, injected so tests can supply a Dio.
final registerServiceProvider = Provider<RegisterService>(
  (ref) => RegisterService(),
);
