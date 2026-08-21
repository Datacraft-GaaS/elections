# abis-svc Architecture

## Components

Four Rust crates, one Django console, one Python vendor service, three shard tiers.

| Component | Language | Role |
|---|---|---|
| `abis-api` | Rust/Axum | REST (`:8080`) and gRPC (`:8443`) gateway; idempotency; outbox; event emission |
| `abis-matcher` | Rust | HNSW-backed 1:N and 1:1 scorer; shard router; vendor-router circuit-breaker |
| `abis-attest` | Rust | TDX/SEV-SNP report verification; nonce registry; HSM signer integration |
| `abis-adjudication` | Rust (planned) | Priority queue worker; case FSM; SLA timer |
| Django console | Python/Django | Investigator UI; case assignment; audit trail |
| `abis-vendor` | Python/FastAPI | Pluggable biometric SDK adapter at `:8090`; vendor-router secondary |

## Gallery Layout

Three shard tiers distribute the 22M+ voter gallery across confidential-compute nodes:

```
Tier 1 (hot)   — top 5M subjects by access frequency (HNSW ef=200)
Tier 2 (warm)  — next 10M subjects (HNSW ef=150)
Tier 3 (cold)  — remainder; lazy-loaded from MinIO snapshot (HNSW ef=100)
```

Each tier runs a minimum of 2 replicas (`abis-matcher` pods). The shard router in `abis-matcher` fans out 1:N queries across all tiers and merges candidate lists before threshold scoring.

## HSM Signer

The `abis-attest` crate calls the HSM signer service (`:8444`, mTLS) for:
- Signing STH entries after enrolment batch commit.
- Countersigning adjudication decisions.
- Verifying TDX/SEV measurement values against golden digest stored in OpenBao.

## Enrolment Flow

```mermaid
sequenceDiagram
    participant Kit as Enrolment Kit
    participant API as abis-api (:8080)
    participant Attest as abis-attest
    participant Kafka
    participant Matcher as abis-matcher
    participant Vendor as abis-vendor (:8090)
    participant PG as PostgreSQL

    Kit->>API: POST /api/v1/abis/match/1n + X-Eel-Attestation
    API->>Attest: verify_report(attestation_header)
    Attest-->>API: Ok(claims)
    API->>PG: INSERT outbox (idempotency_key, payload)
    API-->>Kit: 202 Accepted {job_id}
    API->>Kafka: publish eel.abis.enrolment.submitted.v1
    Kafka->>Matcher: consume eel.abis.enrolment.submitted.v1
    Matcher->>Vendor: extract_template(blob)
    Vendor-->>Matcher: TemplateVector
    Matcher->>Matcher: 1:N search across shard tiers
    alt score in [lower, upper]
        Matcher->>PG: INSERT adjudication_cases
        Matcher->>Kafka: publish eel.abis.adjudication.case.created.v1
    else score < lower
        Matcher->>PG: UPDATE enrolments SET status=clean
        Matcher->>Kafka: publish eel.abis.enrolment.clean.v1
    else score >= upper
        Matcher->>PG: UPDATE enrolments SET status=duplicate
        Matcher->>Kafka: publish eel.abis.enrolment.duplicate.v1
    end
```
