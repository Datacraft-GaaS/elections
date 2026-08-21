# rts-device-api

Module M08 kit-facing API for form submission, SMS fallback, and cryptographic attestation.

Provides low-bandwidth endpoints for polling station kits (M08) to submit Form 34A/B/C. Supports online REST submission and SMS fallback (lossy encoding). Verifies kit attestation (ed25519 signature over form bytes + timestamp) to prevent forgery. Queues forms with replay-prevention (station + timestamp dedup).

Fits into rts-svc as the kit gateway: kits call device-api, device-api queues forms, rts-domain processes them.

## Public API

Exposed modules via lib.rs:
- submission: FormSubmission, submit_form34(form_bytes, kit_sig, timestamp) -> SubmissionReceipt
- sms_fallback: SMSDecoder, decode_sms(message) -> FormPartial
- attestation: AttestationVerifier, verify_kit_signature(form_bytes, kit_id, sig) -> Result
- replay_prevention: ReplayFilter, check_dedup_key(station_id, timestamp)

## Feature flags

None currently.

## Testing

Unit tests (attestation, SMS decoding):
```bash
cargo test --lib
```

Attestation verification tests:
```bash
cargo test attestation::
```

SMS fallback tests (with fixtures):
```bash
cargo test sms_fallback:: --features test-fixtures
```

Integration tests:
```bash
cargo nextest run
```

## Dependencies of note

Workspace:
- rts-domain: Form queuing
- eel-observability-rs: Submission tracing

External:
- ed25519-dalek: Signature verification
- tokio: Async runtime

## Related docs

See services/rts-svc/docs/m08-kit-protocol.md for kit API spec.
See services/rts-svc/docs/sms-fallback-encoding.md for SMS format.
See services/rts-svc/docs/kit-attestation.md for signature verification.
