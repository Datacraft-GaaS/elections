# workforce-svc

Poll-worker lifecycle service: recruitment pipeline, worker registry & contacts, postings & equipment claims, tasking, performance records, interview records, and worker notifications.

Design and domain rules: [docs/workforce-lifecycle.md](../../docs/workforce-lifecycle.md) — this crate implements slice **W3** (§10); the recruitment state machine mirrors §1, contacts §4, postings/equipment §5, tasking §6, interviews §3.3, notifications §4/§8, events §8.

## Layout

- `crates/workforce-api` — single Rust crate exposing the HTTP API (`axum`, store enum `Pg`/`Mem` following the `ecc-api` pattern). Run with `DATABASE_URL` set for Postgres (schema in `migrations/V1__workforce.sql`, `V2__workforce_interviews_notifications.sql`) or unset for an in-memory store. Binds `HTTP_ADDR` (default `0.0.0.0:8100`).
- `migrations/V1__workforce.sql` — tables + indexes for the core bounded contexts.
- `migrations/V2__workforce_interviews_notifications.sql` — interviews, notifications, and the just-in-time training link on task templates.

## Notifications

Worker notifications are recorded against a worker and resolved to the worker's
preferred contact point (hashes only, DPA 2019). `posting` notifications fire
automatically on posting creation, `appointment` on appointment, and `training`
when a task template carrying a `training_content_id` is assigned. `payment`
notifications are posted by payroll (M55) through the generic endpoint:

- `POST /v1/workforce/notifications` — send a worker notification
- `GET  /v1/workforce/notifications?worker_id=` — list a worker's notifications

## Cross-service events (design §8)

Cross-service state leaves this service only via the transactional outbox
(`eel-outbox-rs` → Debezium CDC). Payloads carry ids and hashes, never PII.
Envelope builders live in `crates/workforce-api/src/events.rs`; the API
contract is `packages/openapi-contracts/workforce/openapi.yaml`.

| Topic | Emitted on |
|---|---|
| `eel.workforce.application.stage_changed.v1` | any recruitment stage transition |
| `eel.workforce.worker.appointed.v1` | appointment (worker created) |
| `eel.workforce.posting.activated.v1` | posting created |
| `eel.workforce.equipment.kit_issued.v1` | equipment claim created |
| `eel.workforce.equipment.kit_returned.v1` | equipment returned |
| `eel.workforce.task.completed.v1` | task marked done |
| `eel.workforce.performance.evaluation_recorded.v1` | performance record appended |

## Configuration

| Env var | Default | Meaning |
|---|---|---|
| `DATABASE_URL` | unset → Mem store | Postgres connection string |
| `HTTP_ADDR` | `0.0.0.0:8100` | Listen address |
| `WORKFORCE_WAITLIST_CAPACITY` | `1000` | Per-(county, role) selection capacity before waitlisting |

## Tests

```bash
cargo test -p workforce-api
```
