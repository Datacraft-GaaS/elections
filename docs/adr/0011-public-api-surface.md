# ADR 0011: Public API surface

Status: accepted
Date: 2026-08-21

## Context

Public consumers span millions of concurrent read viewers, mobile apps on lossy Kenyan networks, accredited party agents and observers, media aggregators, researchers pulling bulk datasets, and multi-channel citizen access (SMS/USSD/WhatsApp/IVR). One API tier collapses under this diversity; five tiers with different technology and different SLAs is the honest shape.

## Decision

**Five tiers, each with its own technology and cache posture.**

| tier | traffic | technology | cache |
|---|---|---|---|
| **1. Signed static snapshots** (~90% load) | Cache-heavy, read-only, high-fanout | Rust producer → cosign-signed JSON/CBOR/GeoJSON tarballs → MinIO → nginx+Varnish → ISP edges | long TTL, revalidate on signature version change |
| **2. Live per-user reads** | Low latency, per-request auth-optional | Rust axum + utoipa behind Envoy Gateway + WAF | short TTL where applicable |
| **3. Accredited / authenticated** | Moderate volume, per-caller RBAC | Rust axum with Keycloak JWT + SpiceDB check + audit log | none |
| **4. Bulk data** | Researchers, media, aggregators | Nightly signed Parquet + CSV to MinIO, downloaded via edge | immutable per-version |
| **5. Real-time streams** | SSE (public unauthenticated) + WebSockets (authenticated) | Rust axum SSE/WS, Kafka spine, LiveView Channels for portals | N/A |

**Common conventions across all tiers:**

- **OpenAPI 3.1 spec-first**, schemas in `packages/openapi-contracts/`; generated Dart/TS/Python/Rust clients into `packages/api-clients/`; CI gates on `openapi-diff` and `buf breaking`.
- **Everything consensus-relevant is signed and verifiable**. Cosign signature + Merkle inclusion proof + snapshot ID in response headers. Client SDKs verify automatically.
- **Content negotiation**: JSON default, **CBOR alternative for mobile** (Accept: application/cbor). Zstd for large payloads.
- **Errors as RFC 9457 `application/problem+json`**, never leak stack traces or schema internals.
- **Cache negotiation**: strong `ETag`, `Cache-Control`, `Last-Modified`; cache key includes signature version.
- **Rate limits** with `RateLimit-*` response headers.
- **Idempotency-Key** mandatory on all writes.
- **Versioning** in URL path (`/api/v1/…`).
- **CORS**: strict origin allowlist for browser callers; open for public data endpoints only.

**Explicitly NOT exposed:**

- Django REST / Django Ninja for public: Django is internal admin only.
- GraphQL for public: malicious query construction + N+1 at scale.
- Live DB-query API for bulk consumers: nightly signed files instead.
- LiveView-exposed HTTP JSON for machines: LiveView is UI transport; machines call the same Rust services LiveView uses.

## Consequences

Positive: origin sees <5% of raw public read load; every response is independently verifiable; single canonical source of business logic (Rust services); mobile clients get bandwidth-efficient CBOR.

Negative: five tiers is more surface than one, but each is genuinely different in shape; consolidation would drown one of them. CBOR alongside JSON adds a codec path but the client library handles it transparently.

## Alternatives rejected

- One-tier API for everything: cannot serve 5-10M cached reads and per-user auth from the same layer under adversarial load.
- Django for public: rejected, Django is behind the admin realm.
- GraphQL: attack surface hard to defend at DDoS scale.
- No signing: fails the "everything verifiable" ambition and undermines transparency-log utility.

## Related

ADR-0009 (resilience middleware), ADR-0010 (edge), ADR-0015 (Kafka spine for streams), ADR-0017 (transparency log leaves), ADR-0002 (Rust for API services).
