# ADR 0009: Resilience posture

Status: accepted
Date: 2026-08-21

## Context

Millions of concurrent viewers on election night; sustained adversarial pressure at every layer; a single failing dependency must not cascade. Framework and infrastructure choices alone do not survive election night, the discipline that does is degradation, bulkheading, backpressure, idempotency, and load shedding, coded into shared libraries so no service can accidentally skip them.

## Decision

Every service inherits these from shared crates (`eel-resilience-{rs,py,ex}`, `eel-loadshedding-{rs,py,ex}`) and satisfies them by construction, not by review.

- **Three operating modes per service**, normal / degraded / read-only-lockdown. Feature-flagged, one-command switchable, documented per module in `docs/runbooks/<module>-degradation.md`, rehearsed monthly.
  - Normal: live queries, real-time updates.
  - Degraded: serve last-known-good signed snapshot; disable expensive queries; freeze SSE fan-out to sampled interval.
  - Read-only lockdown: reject all writes, serve cached content, log every request, alert IR immediately.
- **Idempotency mandatory** on every write endpoint. Client-provided `Idempotency-Key` header; server deduplicates by (subject, key) with 24h TTL. Enforced by `eel-api-server-rs` middleware and equivalents.
- **Circuit breakers** on every external call: half-open probe recovery; per-dependency budget; failures open circuit, degraded mode may trigger automatically.
- **Bounded queues everywhere**: no unbounded channels; drop policy documented per queue.
- **Bounded concurrency**: per-upstream `Semaphore` (Rust) / equivalent, plus per-request timeout, plus per-endpoint concurrency cap.
- **Backpressure end-to-end**: Broadway (Elixir) or explicit tokio flow control (Rust) enforces backpressure at every hop; Kafka consumers pause on downstream slowness rather than piling up.
- **Load shedding by priority**: authenticated admin requests > authenticated user > accredited machine > anonymous. Shed lowest priority first when resource envelope exceeded; return `503 Retry-After` with problem+json.
- **Retry budgets**: never unbounded; global per-caller budget; retries only on idempotent operations.
- **Graceful shutdown**: SIGTERM triggers connection drain (30s max), then hard-stop; K8s preStop hooks honour this.
- **Chaos engineering** as first-class deliverable: weekly Chaos Mesh runs in staging + monthly production-equivalent drill (`chaos/scenarios/`). Kill pods, partition sites, fail primary Postgres, saturate links, poison DNS, fill disks, expire certs, revoke credentials.

## Consequences

Positive: no single failing dependency cascades; degraded modes are known and rehearsed, not discovered on election night; retry storms cannot amplify.

Negative: shared-library discipline requires strict review, a service that bypasses `eel-api-server-rs` middleware breaks the guarantees. Enforced by code review + linters + integration tests that assert idempotency on every write endpoint.

## Alternatives rejected

- Per-service ad-hoc resilience: relitigated every module; incompatible failure modes.
- Framework-native circuit breakers (Django/Phoenix): fragmented, not portable across languages; shared library ensures parity.

## Related

ADR-0011 (public API surface enforces middleware), ADR-0010 (edge shares load-shedding conventions), ADR-0018 (write path SLA is what degraded mode protects).
