# M10 — RTS Back-End

**Runs on:** Primary + mirror DC with confidential-compute for ICR workers
**Owner:** Datacraft
**IPR:** Custom Software vests in IEBC per GCC 15.4

![RTS state machine — transitions from device queue through publication and Merkle anchoring, with human-review branch and dispute branch](../../diagrams/architectural/a3-rts-state-machine.svg)

*Figure A3 — RTS State Machine (TLA+-verified). Interactive source: [`../../diagrams/architectural/a3-rts-state-machine.html`](../../diagrams/architectural/a3-rts-state-machine.html)*


## 1. Purpose

Ingest scanned Forms 34A from KIEMS RTS device clients at national scale, verify signatures, run ICR to propose extraction, feed the Returning Officer tally module for human confirmation, aggregate signed Form 34B / 34C, publish signed snapshots to the Public Results Portal, and maintain the cryptographic transparency of every step.

## 2. Personas

- RTS Device (M08) — kit principal
- RO / CRO (M09) — human confirms + signs
- Public (M11) — reads published forms
- Independent Auditor — reads audit trail + escrow mirror
- IEBC Commissioner — publishes national snapshot

## 3. Functional Requirements

- **FR-M10-001** Ingest Form 34A submissions from kits via mTLS with kit-certificate + per-form signature verification.
- **FR-M10-002** Store original signed image immutably (object store); hash-index in database.
- **FR-M10-003** Sharded ingest per constituency to avoid hot spots.
- **FR-M10-004** ICR / AI extraction: OCR handwritten Form 34A → structured 34B / 34C proposal per candidate per position. Confidence score per field.
- **FR-M10-005** ICR runs in confidential compute enclave; model trained on synthetic + historic anonymised only.
- **FR-M10-006** Route low-confidence extraction to human review (M09).
- **FR-M10-007** Random sampling of high-confidence extractions for human quality control.
- **FR-M10-008** Aggregation service produces Form 34B / 34C proposals from confirmed 34As.
- **FR-M10-009** RO / CRO signs via M09; back-end verifies signature + policy (right scope, right personnel).
- **FR-M10-010** Publication service: on IEBC Commissioner authorisation (M-of-N quorum), sign snapshot with HSM + push to CDN + append to transparency log + mirror to escrow.
- **FR-M10-011** Per-elective-position pipelines: President, Governor, Senator, MP, CWR, MCA, plus referenda; independent processing so a failure in one doesn't block another.
- **FR-M10-012** Rate limiting, DDoS shielding, WAF at ingress.
- **FR-M10-013** Immutable event log with cryptographic tamper-evidence.
- **FR-M10-014** SMS/USSD fallback receiver: reconstruct signed Form 34A from SMS submission; alert if signature invalid.
- **FR-M10-015** Independent parallel-tally: a second service replays signed events; any drift raises P0 alert.
- **FR-M10-016** Public-portal feed: signed snapshots pushed to CDN at defined cadence; portal never reads directly from production database.
- **FR-M10-017** Dispute handling: if a form is disputed, its published status is marked; original signed image and audit trail remain available to petitioners.

## 4. Non-Functional Requirements

- **NFR-M10-001** 60,000 concurrent RTS sessions sustained.
- **NFR-M10-002** p99 authorisation ≤ 100 ms.
- **NFR-M10-003** RTO ≤ 30 min; RPO ≤ 5 min.
- **NFR-M10-004** ICR pipeline throughput: 46,000 forms × 6 positions ÷ 4 hours = ~19 forms/sec sustained with headroom to 100 forms/sec.
- **NFR-M10-005** Publication snapshot generation ≤ 5 min per snapshot cycle.
- **NFR-M10-006** Confidential compute attestation on every ICR worker.
- **NFR-M10-007** Formal TLA+ specification for the state machine; no double-count, no lost writes.
- **NFR-M10-008** Active-active HA deployment across primary (Nairobi DC) and mirror site: M10 is a hub module with 20+ downstream dependents; active-passive is insufficient. Both nodes serve write traffic behind a consistent-hashing load balancer. Failover RTO ≤ 30 s. This constraint must appear in the deployment plan and be verified at FAT.

## 5. Integrations

- RTS Device (M08) — ingest
- RTS RO Module (M09) — human loop
- Public Portal (M11) — publication
- HSM (P06)
- Audit + Merkle log + Transparency log (P05)
- Escrow mirror
- Independent parallel-tally service

## 6. Data touched

- `Form34A` (create, sign, publish)
- `Form34B/C` (create, sign, publish)
- `ResultsPublicationSnapshot` (create)
- `MerkleLogRoot` (append)
- `AuditEvent` (append)

## 7. Failure Modes

| Failure | Response |
|---|---|
| Ingest overload | Back-pressure; per-station shard; kits retry with jitter |
| ICR mis-extraction | Human review routed; original image authoritative |
| Signature mismatch | Reject submission; alert war-room |
| Parallel-tally drift | P0 alert; freeze publication until reconciled |
| CDN publication failure | Snapshot re-signed and retried; static fallback available |
| Escrow mirror lag | Alert; publication continues; catch up async |

## 8. Acceptance Criteria

| # | Criterion | Method |
|---|---|---|
| A-M10-01 | 60K concurrent sessions p99 ≤ 100 ms | Load test |
| A-M10-02 | Every published snapshot has Merkle root + escrow mirror entry | Audit trace |
| A-M10-03 | Parallel-tally drift raises P0 alert | Fault-injection |
| A-M10-04 | ICR runs in attested confidential enclave | Attestation report |
| A-M10-05 | SMS/USSD fallback delivers signed Form 34A end-to-end | Field trial |
| A-M10-06 | Formal TLA+ verification pass | TLC model checker |
| A-M10-07 | Publication cycle ≤ 5 min | Bench |

## 9. Cross-References

- `M08-rts-device.md`, `M09-rts-ro.md`, `M11-public-portal.md`
- `../SECURITY.md` §3.7, §3.9
- `../THREAT_MODEL.md` §11–13
