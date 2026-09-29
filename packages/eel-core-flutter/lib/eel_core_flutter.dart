/// Public API of eel_core_flutter.
///
/// Consumers import this, not `src/`. Every app in the monorepo reached into
/// `package:eel_core_flutter/src/...` because no public library existed, which
/// is what the analyzer's implementation_imports lint reports: it means a
/// package's internals are part of its contract, and they are free to change.
library;

export 'src/audit/audit_model.dart';
export 'src/audit/audit_service.dart';
export 'src/biometric/biometric_policy.dart';
export 'src/bootstrap/bootstrap_fsm.dart';
export 'src/connectivity/connectivity_probe.dart';
export 'src/location/location_service.dart';
export 'src/otp/otp_service.dart';
export 'src/outbox/biometric_submission_outbox.dart';
export 'src/upload/evidence_upload.dart';
