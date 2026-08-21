# eel personnel (M33)

Personnel attendance and dispatch app for election staff. Supports 250k concurrent peak users on election day. GPS-tagged check-in/check-out, shift dispatch coordination, and payroll confirmation signature.

## Features

- GPS-tagged attendance tracking (check-in, break, check-out)
- Real-time shift dispatch and assignment
- Payroll confirmation signature
- Offline attendance queue
- Play Integrity attestation
- 11-language UI
- WCAG 2.2 AA accessibility
- Public Store + MDM deployment options

## Build

```bash
cd services/field-mobile-svc/apps/personnel
# Public Store build
flutter build apk --release --flavor production

# MDM-managed build (additional signing)
flutter build apk --release --flavor mdm
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
- `API_BASE_URL`: dispatch-api backend endpoint
- `STAFF_ID`: personnel ID
- `DUTY_STATION_ID`: assigned polling station
- `GPS_ENABLED`: enable location tracking (boolean)

## Device Requirements

- Android SDK 31+
- 1GB RAM minimum
- GPS (for location tracking)
- Public Store (eel-personnel) or MDM enrollment

## Release

**Public Store**: eel-personnel (Google Play Store)
**MDM Track**: internal Google Play internal track for enrolled staff

## Performance SLA

- Check-in transaction: <= 5 seconds
- GPS location capture: <= 90 minutes (M33 SLA)
- Peak concurrent: 250k users (election day)
- p95 response time: <= 2 seconds

## Architecture

- **Frontend**: Flutter + Dart
- **Backend**: gRPC client to field-mobile-svc/crates/dispatch-api
- **Storage**: SQLite encrypted queue for attendance records
- **Attestation**: Android Play Integrity

## Attendance Flow

1. Check-in: capture GPS location, timestamp, and reason (start shift/break end)
2. Local queue: store if offline
3. Sync: transmit when connectivity available
4. Payroll: staff member signs off on daily hours
5. Export: dispatch-api exports to payroll system (HR)

## Dispatch Management

Dispatchers (admin app) can:
- View real-time personnel locations
- Assign staff to stations
- Send urgent messages
- Track attendance patterns

Personnel app receives dispatch assignments and can:
- Accept or decline assignment
- View travel directions to station
- Mark assignment complete

## Location Privacy

- GPS data encrypted in transit (TLS)
- Location history stored locally only (30-day retention)
- Compliance with data protection requirements
- Opt-out per staff member via MDM policy

## Payroll Integration

Daily attendance record includes:
- Check-in and check-out times
- Break duration
- GPS coordinates at check-in
- Staff signature (PIN or biometric if available)

Exported via dispatch-api to support-admin-svc for payroll processing.

## Related Services

- **field-mobile-svc**: dispatch API backend (dispatch-api crate)
- **support-admin-svc**: payroll and HR integration
