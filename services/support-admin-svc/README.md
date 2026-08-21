# support-admin-svc

Administrative backbone service for the IEBC IEMS platform. Covers special voter workflows, intent preservation, budget and expenditure, HR and payroll for ~250,000 temporary election staff, and the 15-year long-term archive.

| Field | Value |
|---|---|
| Modules | M25 special-voter-workflows, M27 intent-preservation, M54 budget-expenditure, M55 hr-payroll, M69 long-term-archive |
| Tier | T2 |
| Availability SLO | 99.9% |
| RTO | 4 h |
| RPO | 1 h |
| Key SLO: payroll run | 250,000 staff processed in < 4 h |
| Key SLO: archive ingest | > 1 GB/min sustained |

## Architecture overview

```
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ special-voter│  │    intent    │  │    payroll   │
│  (M25)       │  │  (M27)       │  │  (M55)       │
└──────┬───────┘  └──────┬───────┘  └──────┬───────┘
       │                 │                 │
       └────────┬────────┘                 │
                │         ┌────────────────┘
                ▼         ▼
          PostgreSQL 17 (schema-per-module)
          Kafka (eel-outbox)
          ┌──────────────┐  ┌──────────────┐
          │    budget    │  │   archive    │
          │   (M54)      │  │   (M69)      │
          └──────────────┘  └──────────────┘
```

**State stores**: PostgreSQL 17 (voter facilities, payroll runs, archive collections, budget ledger), MinIO (archive object store), Valkey (payroll idempotency cache).

**Upstream deps**: bvr-svc (register extract for special voter identification), rts-svc (form34 events for archive ingest), field-mobile-svc (attendance events for payroll trigger), transparency-svc (tlog inclusion proofs for archive attestation).

**Downstream consumers**: OAG (M59 evidence packages from archive), IFMIS/PPRA (budget vouchers), M-Pesa B2C (payroll disbursement), KNADS/DataCite (archive preservation).

## Directory layout

```
support-admin-svc/
  Cargo.lock, Cargo.toml   workspace root
  crates/
    archive/               M69 long-term archive
    budget/                M54 budget and expenditure
    intent/                M27 intent preservation
    payroll/               M55 HR and payroll
    special-voter/         M25 special voter workflows
  deploy/
    base/                  Kustomize base manifests
    primary/               Anniversary Towers overlay
    mirror/                Bomas overlay
  django/
    finance/               Budget admin (IFMIS UI)
    hr/                    HR and payroll admin
    support-admin/         General admin interface
  docs/
  migrations/              PG schema migrations
  py/
    bagit/                 BagIt archive packaging (M69)
    ifmis/                 IFMIS integration adapter (M54)
    mpesa-b2c/             M-Pesa B2C disbursement batch (M55)
  src/                     main Rust binary
  target/
  tests/                   integration tests
```

## Key APIs

OpenAPI spec: `packages/openapi-contracts/support-admin/`

| Method | Path | Module | Purpose |
|---|---|---|---|
| GET | /v1/support/special-voters/{facility_id} | M25 | Voters at a special facility |
| POST | /v1/support/special-voters/events | M25 | Record identification event |
| POST | /v1/support/intent/pre-mark | M27 | Pre-mark ballot validation |
| GET | /v1/support/intent/spoilt-ballots/{station_id} | M27 | Spoilt ballot query |
| POST | /v1/support/payroll/runs | M55 | Trigger payroll run |
| GET | /v1/support/payroll/runs/{id} | M55 | Run status |
| POST | /v1/support/budget/vouchers | M54 | Create payment voucher |
| GET | /v1/support/archive/collections | M69 | List archive collections |
| POST | /v1/support/archive/ingest | M69 | Ingest election records |

## Key events

**Emitted**:
- `support.payroll.run.completed` (M55)
- `support.archive.collection.ingested` (M69)
- `support.special.voter.enrolled` (M25)
- `support.intent.spoilt.adjudicated` (M27)
- `support.budget.voucher.approved` (M54)

**Consumed**:
- `field-mobile.attendance.checked-in` (payroll trigger)
- `rts.form34.published` (archive ingest)
- `bvr.voter.enrolled` (special voter cross-check)
- `transparency.sth.published` (archive attestation)

## Data model

- `migrations/` contains versioned PG migrations for each module schema.
- `data_dictionary.yaml` classifies every column (PIIRestricted, PIISensitive, PIIPublic).
- Key tables: `special_voter_facilities`, `special_voter_events`, `intent_events`, `spoilt_ballot_adjudications`, `staff_contracts`, `payroll_runs`, `payroll_disbursements`, `budget_lines`, `payment_vouchers`, `archive_collections`, `ingest_jobs`, `preservation_attestations`.

## Building and running

```bash
# Rust crates
cargo nextest run -p support-admin-svc

# Python adapters
uv run pytest -vxs services/support-admin-svc/py/tests/ci

# Django admin
cd services/support-admin-svc/django/hr
uv run python manage.py runserver

# Full stack (dev)
just up
just seed
```

## Testing

| Layer | Command | SLA target |
|---|---|---|
| Unit | `cargo nextest run --lib` | < 60s |
| Property | `cargo nextest run --test proptest` | < 120s |
| Integration | `cargo nextest run --test integration` | < 5 min |
| Contract | `just contracts` | < 2 min |
| Load: payroll | `k6 run services/support-admin-svc/loadtest/payroll-250k.js` | 250k runs in < 4 h |

## Deployment

Kustomize overlays at `services/support-admin-svc/deploy/`:

```bash
kubectl apply -k services/support-admin-svc/deploy/primary
kubectl apply -k services/support-admin-svc/deploy/mirror
```

ArgoCD ApplicationSet: `platform/argocd/appsets/support-admin-svc.yaml`. HPA: 2-10 replicas based on CPU and queue depth. PDB: min 1 available.

## Observability

- Grafana dashboard: `platform/grafana/dashboards/support-admin-svc.json`
- Key metrics: `eel_payroll_run_duration_seconds`, `eel_archive_ingest_bytes_total`, `eel_special_voter_events_total`, `eel_budget_voucher_approved_total`
- Alerts: payroll run > 3.5 h, archive lag > 10 min, IFMIS error rate > 1%, M-Pesa B2C failure rate > 0.1%
- SLO dashboard: `platform/grafana/dashboards/support-admin-slo.json`

## Runbooks

- Degradation: `docs/runbooks/support-admin-svc-degradation.md`
- Payroll incident: `docs/runbooks/support-admin-svc-payroll-incident.md`
- Archive recovery: `docs/runbooks/support-admin-svc-archive-recovery.md`

## Degradation contract

| Mode | Behaviour |
|---|---|
| Normal | All workflows live: special voter identification, payroll, budget, archive ingest |
| Degraded | Payroll disbursement deferred; special voter identification from cached register extract; IFMIS sync queued |
| Read-only | Archive read-only; payroll and budget halted; special voter identification from last snapshot |

## Chaos scenarios

- `chaos/support-admin-svc/`: M-Pesa B2C rate limit simulation, IFMIS connection loss, archive MinIO unavailability, payroll DB primary failover during run.

## Statutes satisfied

- Elections Act §§53-55 spoilt ballots (M27 intent preservation)
- Kenya Data Protection Act 2019 §40 right to erasure (M69 destruction certificates)
- Kenya Data Protection Act 2019 §71 cross-border transfer controls (diaspora staff)
- KNADS Act 15-year retention obligation (M69)
- SoD controls per PPRA procurement rules (M54)

## Follow-ups

- M53 voter-id-card scope activation gate: coordinate with bvr-svc on NRB MoU status.
- M55 NHIF/SHIF statutory deduction rate updates: track NHIF reform effective dates.
- M69 format migration schedule: define JHOVE validation triggers for archive refresh.
