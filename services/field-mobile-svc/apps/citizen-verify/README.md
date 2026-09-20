# Citizen Verify App — OTP Integration Update

## OTP Service Integration

The citizen-verify app now uses the real `OtpService.sendOtp(phone)` method from `packages/eel-core-flutter/lib/src/otp/otp_service.dart`. This connects directly to the telco-sms adapter for actual SMS/voice OTP delivery.

### Changes

- **OtpService.sendOtp()**: Sends OTP via telco-sms adapter, emits M5 audit event, returns success/failure.
- **otp_screen.dart**: Updated to call OtpService, handle response, show loading/error states.
- **M5 audit trail**: Every OTP attempt emits auditId, proofHash, signature, correlation_key.

### Usage

```dart
final response = await OtpService.sendOtp('+254701234567');
if (response.success) {
  // OTP sent successfully
} else {
  // Handle error
}
```

### Dependencies

- `packages/eel-core-flutter/lib/src/otp/otp_service.dart`
- `packages/eel-core-flutter/lib/src/audit/audit_service.dart`
- `packages/eel-core-flutter/lib/src/audit/audit_model.dart`
