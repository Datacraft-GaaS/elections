# eel party-agent (M30)

Political party agent companion app for election day. Allows accredited party agents at polling stations to countersign tallies, view real-time results feed, and raise objections. Requires MDM provisioning and HSM-backed cryptographic signing.

## Features

- Tally countersignature with HSM-backed key
- Real-time results feed (authenticated, party-only)
- Objection submission and tracking
- Form 34A verification and sign-off
- Offline queue for countersignatures (7-day retention)
- Play Integrity attestation + Titan M HSM signature
- 11-language UI
- WCAG 2.2 AA accessibility
- Party agent authentication and authorization

## Build

```bash
cd services/field-mobile-svc/apps/party-agent
flutter build apk --release --flavor production
flutter build apk --release --flavor staging
```

## Development

```bash
flutter pub get
flutter run --flavor development
```

## Tests

```bash
flutter test
flutter test integration_test/
flutter test --machine test/ | patrol
```

## Environment

Configure via `assets/config/`:
- `API_BASE_URL`: field-mobile-svc backend endpoint
- `PARTY_AGENT_ID`: unique agent identifier
- `HSM_KEY_ALIAS`: HSM key reference (Titan M)

## Device Requirements

- Android SDK 33+
- 2GB RAM minimum
- Titan M secure element (for HSM signing)
- MDM enrollment via Jamf Mobile (policy: eel-party-agent-mdm.json)

## Release

Internal Google Play track (eel_party_agent).

## HSM Signing

Uses Titan M secure element to store party agent private keys. Signatures are non-repudiable and immune to malware extraction.

```bash
# Key provisioning (admin only)
flutter run --flag "provision-hsm-key"
```

## Architecture

- **Frontend**: Flutter + Dart
- **Backend**: gRPC client to field-mobile-svc/crates/mobile-bff
- **Storage**: SQLite encrypted queue for countersignatures
- **Attestation**: Android Play Integrity + Titan M signature

## Data Flow

1. Tally receipt: polling station staff present Form 34A
2. Verification: validate form structure and KIT signature
3. Countersignature: sign with HSM key (requires PIN entry)
4. Queue or transmit: store locally or send via network
5. Sync: drain queue on reconnect

## Objection Workflow

1. Agent logs into app with party credentials
2. Identifies issue at polling station
3. Submits objection with evidence (photo, description)
4. Receives objection ID and escalation path
5. Can view status updates in real-time feed

## Results Feed

Real-time results stream authenticated via OAuth2:
- Polling station tallies as they arrive
- Provisional constituency-level aggregates
- Objection impact on results

## Security

- HSM key never leaves device
- Signatures are non-exportable
- Device attestation mandatory
- Rate limiting on results feed access

## Related Services

- **field-mobile-svc**: mobile BFF backend (mobile-bff crate)
- **rts-svc**: tally publication and verification
- **bvi-svc**: event log
