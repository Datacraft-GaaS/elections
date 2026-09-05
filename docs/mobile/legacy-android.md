# legacy-android (WP-3.8 shim) — Decision Plan

Master plan: `docs/mobile/plan.md`. Current: 3 lib files; buttons
`onPressed: () {}` (dead); PARITY.md documents intended coverage. Est: 1 ED.

## Decision (recommended)

Keep as an intentional **shim**, not an app: it must either (a) deep-link
into the real bvr-kit/bvi-kit flows when both are installed, or (b) present
an explicit "install full kit app" guidance screen. Do not implement
parallel features.

## Work items

1. Buttons deep-link: `eel://kit/bvr/enrol` and `eel://kit/bvi/identify`
   (scheme registered in bvr-kit/bvi-kit Android manifests); if apps absent →
   install-guidance screen (MDM managed-app store link).
2. Attestation banner stays (software-only honesty); version floor enforced.
3. PARITY.md rows gain status column values matching reality; CI scanner
   green.

## Testing

- Widget: button paths + deep-link fallback states.
- Integration (device): deep-link lands in target kit screen (2 devices).

## Definition of done

No dead controls; PARITY.md accurate; shim contract documented in service
README.Updating docs/mobile/legacy-android.md -> LEGACY-ANDROID (expanded reference with functions, capabilities, design-system usage, testing)

---
# Audit Reference (5 axes: where / when / who / why / how)
Every action now logs via `AuditService` (encrypted Isar + outbox replay):
- **where**: GPS (LocationService) or station-bound coordinate
- **when**: ISO 8601 UTC timestamp (per entry UUID7)
- **who**: Actor identity + SpiceDB role (actorId, actorRole)
- **why**: Justification / reason / dispute reason (reason field)
- **how**: Method (biometric_attestation_fingerprint / iris / HSM_Ed25519 / OTP / PIN) + SHA-256 proof hash + Ed25519 signature
Offline events queued via outbox; replayed on reconnect with full 5-axis verification.
