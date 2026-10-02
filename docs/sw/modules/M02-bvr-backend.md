# M02 — BVR Back-End (Voter Registration Central System)

**Runs on:** Primary + mirror data centres
**Owner:** Datacraft
**IPR:** Custom Software vests in the Commission per GCC 15.4

## 1. Purpose

Central authoritative store of the Kenya voter register. Ingests enrolment records from BVR device clients, orchestrates ABIS dedup and adjudication, publishes the register at each electoral scope, exposes back-office tools for Returning Officers, and integrates with BI + IPRS.

## 2. Personas

- Enrolment Clerk (via BVR device — write path)
- Returning Officer (back-office reads + status changes)
- the Commission Administrator (bulk operations, exports)
- Auditor (read + audit-trail exports)
- ODPC Officer (compliance queries)

## 3. Functional Requirements

- **FR-M02-001** Ingest enrolment records from BVR device clients via mTLS + kit-certificate authentication.
- **FR-M02-002** Verify per-record signature + digital-certificate binding to kit.
- **FR-M02-003** Store record in encrypted persistent store (register core); biometric templates in field-encrypted column with separate key.
- **FR-M02-004** Enqueue ABIS 1:N dedup check on ingest.
- **FR-M02-005** ABIS returns match / no-match / below-quality → adjudication workflow assigns to investigator queue.
- **FR-M02-006** Investigator UI: side-by-side comparison, minutiae overlay, decision logging, biometric-authenticated approval.
- **FR-M02-007** IPRS integration: on ingest, verify ID against national register; cache last-known status; fail-closed if IPRS down > 24h (blocks new enrolments); log any discrepancy for investigation.
- **FR-M02-008** RO back-office: mark voter status ACTIVE / DECEASED / DELETED / TRANSFERRED / SUSPENDED with biometric authorisation + reason code; every mutation writes a versioned row.
- **FR-M02-009** Voter transfer: authenticated by RO + originating voter fingerprint; polling-station reassigned.
- **FR-M02-010** Gazetted cut-off enforcement: on cut-off date, register freezes for the specific election; further writes blocked; frozen snapshot published to escrow mirror.
- **FR-M02-011** Register generation at any scope (station / ward / constituency / county / national), preliminary or final, in PDF + CSV.
- **FR-M02-012** Bulk export: PDF + CSV of register scope-restricted; requires dual-control approval + audit entry.
- **FR-M02-013** ABIS-triggered image / alphanumeric / fingerprint investigations queue with resolution workflow.
- **FR-M02-014** Electoral-area mapping: propagate polling-station / ward / constituency / county changes to kits via MDM channel.
- **FR-M02-015** Configurable max voters per polling station (default 700).
- **FR-M02-016** Voter streaming / sorting per station for register generation.
- **FR-M02-017** QR / barcode generation per voter record for BVI-day lookup.
- **FR-M02-018** Per-kit monitoring: SNMP + email + dashboard alerts on processes / logs / disk / memory anomalies.
- **FR-M02-019** BI mart publication (async) to the Commission Power BI.
- **FR-M02-020** ODPC subject-access-request workflow: retrieve, rectify, erase, port per Kenya DPA.
- **FR-M02-021** Retention scheduler: enforces contract + 5-year retention on voter data.
- **FR-M02-022** Every state change signed by originator + countersigned by back-end HSM + timestamped + Merkle-logged + escrow-mirrored.

## 4. Non-Functional Requirements

- **NFR-M02-001** 30M voter records + biometric templates + audit history in Kenya-resident storage.
- **NFR-M02-002** RO back-office p99 response ≤ 500 ms for common queries.
- **NFR-M02-003** Ingest throughput sustainable to 1,000 records/sec across the fleet.
- **NFR-M02-004** Register generation for 30M records ≤ 30 minutes (background job).
- **NFR-M02-005** 99.99% availability; RTO ≤ 4 h; RPO ≤ 15 min.
- **NFR-M02-006** Field-encrypted biometric templates with separate keys; confidential compute for matching workers.
- **NFR-M02-007** Bulk export DLP: > 1,000 rows requires dual-control + audit + rate-limit.

## 5. Integrations

- BVR Device (M01) — ingest
- ABIS (M03) — dedup
- IPRS (I01) — identity verification
- BI Power BI (I04)
- Audit / Merkle log (P05)
- HSM (I07 / P06)
- Retention scheduler (C01)
- MDM (O01) — electoral-area propagation

## 6. Data touched

- `Voter` (write, versioned)
- `BiometricTemplate` (write, encrypted)
- `IDDocumentScan` (write)
- `AuditEvent` (write)
- `MerkleLogRoot` (append)

## 7. Failure Modes

| Failure | Response |
|---|---|
| IPRS down > 24h | Fail-closed on new enrolments; cache reads |
| ABIS backlog | Prioritise most-recent; alert war-room |
| Storage full | Auto-scale + alert; refuse-fast writes if full |
| Insider bulk export attempt | Blocked at DLP; alert to internal audit |
| Duplicate detected post-ingest | Move to adjudication queue; do NOT auto-delete |

## 8. Acceptance Criteria

| # | Criterion | Method |
|---|---|---|
| A-M02-01 | Ingest 1,000 records/sec sustained 10 min without loss | Load test |
| A-M02-02 | ABIS-triggered investigation loop closes end-to-end | UAT |
| A-M02-03 | IPRS failure fails-closed on new enrolments; open on reads | Chaos exercise |
| A-M02-04 | Register generation 30M records ≤ 30 min | Bench |
| A-M02-05 | Bulk export > 1,000 rows requires dual approval | Policy test |
| A-M02-06 | Every state change signed + Merkle-logged + escrow-mirrored | Audit trace |
| A-M02-07 | SAR fulfilled within 30 days | Workflow test |
| A-M02-08 | Gazetted cut-off freezes writes and snapshots to escrow | Time-based test |

## 9. Cross-References

- `M01-bvr-device.md` — ingest client
- `M03-abis.md` — dedup engine
- `../SECURITY.md` §3.1, §3.8, §3.9
- `../DATA-MODEL.md` §1, §7
