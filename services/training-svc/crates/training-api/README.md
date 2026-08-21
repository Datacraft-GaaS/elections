# eel-training-api

Unified REST API for all training service modules: LMS enrollment, course completion, accreditation review, and medical incident reporting.

## What it does

Single axum service routing to five module handlers: CMS course catalog (M37), LMS enrollment and completion (M38), media accreditation (M42), observer accreditation (M43), and medical incident reporting (M60). All aggregates stored in PostgreSQL with Kafka event publishing via outbox pattern. Supports offline course completion via Flutter app with local event queueing and sync-on-reconnection.

Fits in training-svc as the primary API surface. Integrates with Django admin (content management), Flutter apps (learner training, field medical reporting), and badge printer firmware (QR code generation).

## Public API

Resource aggregates:

- `eel_training_api::course::Course`: `fn get_lessons(&self) -> Vec<Lesson>`, `fn get_translations(&self, lang_code: &str) -> Option<CourseTranslation>`
- `eel_training_api::enrollment::Enrollment`: `fn complete_lesson(&mut self, lesson_id: uuid::Uuid) -> Result<()>`, `fn is_complete(&self) -> bool`
- `eel_training_api::accreditation::Accreditation`: `fn request_review(&mut self) -> Result<()>`, `fn issue_badge(&mut self, template: BadgeTemplate) -> Result<QrBadge>`
- `eel_training_api::medical::MedicalIncident`: `fn dispatch_to_nearest_facility(&mut self, location: GeoPoint) -> Result<DispatchOrder>`

Endpoints match OpenAPI spec; all return JSON with `error` field on failure.

## Feature flags

`#[cfg(feature = "flutter-sync")]` for offline completion sync protocol; tests use online-only mode.

`#[cfg(feature = "badge-printing")]` for live badge printer firmware integration; tests stub with in-memory badge generation.

## Test commands

```bash
cargo test -p eel-training-api --all-features
cargo test -p eel-training-api --lib course_multilang
cargo test -p eel-training-api --test integration_training -- --include-ignored
```

Mock MCK, KRCS, MoH APIs via pytest-httpserver; verify offline sync via local SQLite fixture.

## Bench commands

```bash
cargo bench -p eel-training-api course_lesson_fetch
cargo bench -p eel-training-api accreditation_badge_issue
```

Course fetch and badge issuance should each complete in < 100ms.

## Dependencies of note

- `uuid6`: course, enrollment, accreditation, incident IDs
- `tokio`: async API calls to Django admin, badge printer, dispatch systems
- `serde_json`: course and incident metadata parsing
- `chrono`: enrollment and incident timestamps

## Related docs

- `docs/api-reference.md` (all endpoint contracts)
- `docs/architecture.md#offline-sync` (Flutter offline completion sync protocol)
- `migration_001_training.sql` (schema: courses, enrollments, accreditations, incidents, translations)
