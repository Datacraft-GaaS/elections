# eel observer (M34)

Accredited observer mobile app for election monitoring. Allows observers to log visit records at polling stations, view public results feed, and submit structured observation reports. MDM-provisioned by the Commission.

## Features

- Polling station visit logging (check-in, check-out, timestamps)
- Observation report submission (structured forms, photo/video evidence)
- Public results feed access (read-only, non-enumerable)
- Observer accreditation verification
- 7-day offline queue for visit logs
- Play Integrity attestation
- 2-language UI (English, Swahili for international observers)
- WCAG 2.2 AA accessibility

## Build

```bash
cd services/field-mobile-svc/apps/observer
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
```

## Environment

Configure via `assets/config/`:
- `API_BASE_URL`: field-mobile-svc backend endpoint
- `OBSERVER_ACCRED_ID`: accreditation credential ID

## Device Requirements

- Android SDK 33+
- 1GB RAM minimum
- Camera (for evidence capture)
- MDM enrollment via Jamf Mobile

## Release

Internal Google Play track (eel_observer).

## Architecture

- **Frontend**: Flutter + Dart
- **Backend**: gRPC client to field-mobile-svc/crates/mobile-bff
- **Storage**: SQLite encrypted queue for visit logs
- **Attestation**: Android Play Integrity

## Visit Logging

Each visit record includes:
- Polling station ID and name
- Observer name and accreditation ID
- Check-in and check-out timestamps
- GPS coordinates (if available)
- Notes and observations

## Observation Reports

Structured reports capture:
- Report category (irregularity, commendation, concern)
- Description and evidence (photos/video)
- Affected parties (if applicable)
- Urgency level
- Submission timestamp

## Results Feed

Public, read-only results stream includes:
- Polling station tallies (non-real-time, delayed publication)
- Constituency aggregates
- National-level results
- Verification proofs (Merkle inclusion)

No enumeration attacks possible; results served via CDN with rate limiting.

## Data Flow

1. Observer accreditation: verify credentials with training-svc
2. Visit logging: local record of station visits
3. Report submission: structured form with evidence
4. Sync: transmit queue on network reconnect
5. Results: fetch public feed via CDN

## Related Services

- **field-mobile-svc**: mobile BFF backend (mobile-bff crate)
- **training-svc**: observer accreditation verification
- **portal-svc**: public results feed
