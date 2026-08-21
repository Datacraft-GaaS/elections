# portal-svc

## Purpose

The public-facing election information portal: displays signed Form 34* results with Merkle inclusion proofs, hosts the voter roll inspection interface, publishes the statutory election calendar and gazette notices, and delivers multilingual referendum explainers with WCAG AAA compliance and TTS support. It is the primary transparency surface for citizens, media, and observers.

## Modules Owned

| M-ID | Slug | Description |
|---|---|---|
| M11 | public-portal | Published forms, Merkle inclusion proofs, public result feed |
| M52 | election-calendar-gazette | Calendar events, statutory rule engine, gazette notices, GovPrinter interface |
| M64 | voter-list-inspection | Roll snapshots, objections, correction claims, auto-referral (T-60) |
| M68 | referendum-explainer | Referendum question explainers, KSL video, TTS, WCAG AAA |

## Key Aggregates and State Stores

- `PublishedForm` (PostgreSQL + CDN): Form 34* records with cryptographic proofs.
- `MerkleRoot` (PostgreSQL): anchored roots from rts-svc.
- `InclusionProof` (generated on-demand): Merkle branch proofs per form.
- `CalendarEvent` (PostgreSQL): statutory election calendar.
- `GazetteNotice` (PostgreSQL): signed and published legal notices.
- `RollSnapshot` (PostgreSQL + object store): voter roll inspection snapshots.
- `Objection` (PostgreSQL): voter inspection objections pending determination.

## Service Dependencies

| Direction | Service | Reason |
|---|---|---|
| Upstream | rts-svc | Published form data and Merkle roots |
| Upstream | bvr-svc | Voter roll snapshots for inspection |
| Upstream | transparency-svc | Bundle download links |
| Upstream | governance-svc | Referendum and by-election calendar events |
| External | GovPrinter | Gazette notice submission |
| Downstream | ecc-svc | Objection referral events |

## Events Emitted

| Event | Topic | Description |
|---|---|---|
| `eel.portal.form.published` | `eel.portal` | Form made publicly available on portal |
| `eel.portal.gazette.published` | `eel.portal` | Gazette notice published |
| `eel.portal.objection.filed` | `eel.portal` | Voter inspection objection received |
| `eel.portal.explainer.published` | `eel.portal` | Referendum explainer content published |

## Events Consumed

| Event | Producer |
|---|---|
| `eel.rts.form34.published` | rts-svc |
| `eel.rts.merkle_root.anchored` | rts-svc |
| `eel.bvr.register.snapshot.published` | bvr-svc |
| `eel.governance.referendum.configured` | governance-svc |

## Primary APIs

- `packages/openapi-contracts/portal/openapi.yaml` - public result query, inclusion proof, roll inspection
- `packages/openapi-contracts/gazette/openapi.yaml` - gazette and calendar
- `packages/openapi-contracts/inspection/openapi.yaml` - voter list inspection
- `packages/openapi-contracts/explainer/openapi.yaml` - referendum explainer content

## SLA Targets (ADR-0018, ADR-0022)

| Metric | Target |
|---|---|
| Form query P99 | less than 200 ms (CDN-cached) |
| Roll inspection query P99 | less than 500 ms |
| Availability | 99.9% |
| Petition-window peak readiness | handled via CDN offload |
| RTO | 15 min |
| RPO | 1 h (CDN can serve stale during failover) |

## Degradation Modes

- CDN serves cached forms if origin is unavailable; stale-while-revalidate up to 5 min.
- Roll inspection objections queue if database write is unavailable.
- GovPrinter API failures delay gazette publication but do not block read path.

## Runbook Links

- [docs/runbooks/degradation-per-service.md](../../runbooks/degradation-per-service.md)
- [docs/runbooks/incident-response.md](../../runbooks/incident-response.md)
