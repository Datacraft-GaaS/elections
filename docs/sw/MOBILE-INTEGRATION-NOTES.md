# Mobile Integration Notes

**Date:** 2026-08-21
**Author:** Datacraft Solutions Architecture
**Status:** Working document — pending stitching-pass review

This file records the integration decisions made when adding the six mobile apps from `mobile/` into the `sw/` specification. It is a companion to the six new modules M29-M34 and the four `sw/mobile/` companion docs.

---

## 1. What Was Integrated

Six mobile app specifications from `mobile/apps/` were imported into the sw/ module namespace as M29 through M34:

| Source file | sw/ Module | App name | Users |
|---|---|---|---|
| `mobile/apps/MA01-iebc-verify.md` | M29 | IEBC Verify (Citizen) | ~35M reachable Kenyans |
| `mobile/apps/MA02-agent.md` | M30 | Agent (Party Agent) | ~20K agents on election day |
| `mobile/apps/MA03-poll-clerk.md` | M31 | Poll Clerk Companion | ~92K Deputy POs + clerks |
| `mobile/apps/MA04-field-support.md` | M32 | Field Support | ~200 Datacraft engineers |
| `mobile/apps/MA05-personnel-dispatch.md` | M33 | IEBC Dispatch (Personnel) | ~250K polling-day workers |
| `mobile/apps/MA06-observer.md` | M34 | Observer | ~500 accredited observers |

Four cross-cutting mobile docs were adapted into `sw/mobile/`:

| Source | Destination |
|---|---|
| `mobile/PRD.md` | `sw/mobile/PRD.md` |
| `mobile/ARCHITECTURE.md` | `sw/mobile/ARCHITECTURE.md` |
| `mobile/SECURITY.md` | `sw/mobile/SECURITY.md` |
| `mobile/ACCEPTANCE-CRITERIA.md` | `sw/mobile/ACCEPTANCE-CRITERIA.md` |

One integration-notes doc was created:

| File | Purpose |
|---|---|
| `sw/MOBILE-INTEGRATION-NOTES.md` | This file — integration decisions, overlap map, open questions |

Source mobile/ files were NOT modified. They remain the source of truth pending stitching-pass consolidation.

---

## 2. Renaming Convention

Inside the six new M29-M34 files, internal requirement IDs were renumbered from MA0X-NNN to M2X-NNN. Acceptance criteria IDs were renumbered from A-MA0X-NN to A-M2X-NN. No changes were made to the source `mobile/` files.

| Old prefix | New prefix |
|---|---|
| FR-MA01- | FR-M29- |
| FR-MA02- | FR-M30- |
| FR-MA03- | FR-M31- |
| FR-MA04- | FR-M32- |
| FR-MA05- | FR-M33- |
| FR-MA06- | FR-M34- |
| NFR-MA01- | NFR-M29- |
| A-MA01- | A-M29- |
| ... etc | ... etc |

---

## 3. Overlap Map and Integration Decisions

### 3.1 M29 Citizen Verify vs M13 Public Verifier App

**Overlap level:** Heavy.

M13 is the multi-channel superset: short-code, SMS, WhatsApp, and PWA paths for verifying polling-station results. M29 is the native-mobile, IEBC-branded app targeting the widest handset base (Android 8+, including budget devices under KES 10,000) with full offline-first capabilities, biometric unlock, and a richer UX.

**Decision:** Keep both as distinct modules for now. M29 is the native-mobile implementation; M13 is the multi-channel superset. They share the same back-end Merkle-proof API (via M10 and M11) and the same Rust FFI verifier logic.

**Stitching-pass recommendation:** Extract the common Rust FFI verifier into a standalone shared library (`libiebc-verifier`) referenced by both M13 and M29. The stitching pass should also decide whether the PWA fallback for M29 is served by the same codebase as M13's PWA or is separately built. They may consolidate at that point.

### 3.2 M34 Observer vs M15 IAB Dashboard

**Overlap level:** Partial — same data source, different access tier.

M15 is for named Independent Assurance Board members who have elevated privileges: they can see pre-publication aggregates and participate in the quorum-ceremony ceremony. M34 is for field observer-mission personnel (domestic civil society, international observers, academic, media) with read-only access to the same public results data the portal exposes, plus observer-specific incident reporting and station visit logs.

**Decision:** Keep separate. Distinct roles, distinct accreditation databases, distinct data-access tiers. M34 does not have IAB role access. M15 does not have the field observation reporting flows M34 provides.

**Stitching-pass note:** The Auditor-General mirror feed from M34 and the RLA sampling feed from M34→M21 should be cross-referenced in M15's escalation path, but the apps remain distinct.

### 3.3 M30 Party Agent

**Overlap level:** None with M01-M28.

M30 introduces a new capability: party-agent countersignature on Form 34A at the polling-station level, attached to the same signed-events chain used by KIEMS. There is no existing module covering this. The RTS back-end (M10) already supports agent countersignature records as a data path; M30 is the client that uses it.

**Decision:** New capability. No consolidation needed.

**Stitching-pass note:** M10 (RTS Backend) specification should be checked to confirm the agent countersignature submission endpoint is documented in the API catalog (`sw/API-CATALOG.md`). If not, add it.

### 3.4 M31 Poll Clerk Companion vs M05 BVI Device

**Overlap level:** UX layer only — M31 mirrors the M05/M06 dashboard in read-only mode.

M05 is the primary device-side BVI software on the ruggedised KIEMS kit. M31 is a companion app on the Deputy PO's own phone that reads the same BVI back-end (M06) in read-only mode. They are distinct by design: M05 is authoritative for voter authentication; M31 cannot authenticate voters.

**Decision:** Keep separate. Distinct deployment context (KIEMS device vs Android/iOS handset), distinct function (authoritative vs companion).

**Stitching-pass note:** The read-only API that M31 uses from M06 should be documented in the API catalog. Confirm M06 exposes a rate-limited, read-only dashboard endpoint for companion apps.

### 3.5 M32 Field Support vs M20 Inventory Management

**Overlap level:** Consumer-service relationship.

M20 (Inventory Management System) is the back-end system of record for kit inventory, spare parts, and station assignments. M32 (Field Support app) is a mobile consumer of M20's IMS API: it looks up spares availability and locates the nearest spare-holding constituency store. There is no overlap in logic; M32 calls M20.

**Decision:** Keep separate. M32 is a client; M20 is a service. The dependency is documented in M32 §5 and §9.

**Stitching-pass note:** M20's specification should confirm it exposes a mobile-friendly API (paginated, offline-cacheable) for M32. If M20 was written by the concurrent agent, check that it covers the spares-availability query M32 needs.

### 3.6 M33 Personnel Dispatch vs M16 Election Command Centre

**Overlap level:** Operator-worker relationship.

M16 (Election Command Centre) is the war-room dashboard that ECC operators use to monitor the election in real time and task field teams. M33 (IEBC Dispatch) is the worker-side app that receives assignments, records check-ins, and reports on-site status. ECC operators in M16 see the M33 attendance feed.

**Decision:** Keep separate. M16 is the command-side tool for ECC operators; M33 is the field-side tool for the 250K workers. They communicate via the Personnel Dispatch back-end (O05).

**Stitching-pass note:** M16's specification should confirm it ingests the M33 attendance feed. The O05 Personnel Dispatch back-end (extended from MeGuard) is referenced in M33 §5 but may not yet be fully specified as a distinct module or as an O-series component. Flag for stitching.

---

## 4. Technology Decisions That Apply Across M29-M34

| Decision | Rationale |
|---|---|
| Flutter 3.x for all six apps | Single codebase, IEBC-teachable, Datacraft house standard (CrowdAfrica, MeGuard), offline-first primitives, native camera/scanner |
| Rust FFI for crypto-critical modules | Audited, deterministic, no GC pauses; shared across all six apps as libiebc-verifier |
| Riverpod for state management | Compile-time-safe, testable, Datacraft standard |
| Isar + SQLCipher for local store | Encrypted at rest, schema-versioned, performant offline |
| dio + mTLS interceptor chain | Unified retry, backoff, telemetry, certificate pinning |
| WCAG 2.2 AA (not 2.1) | Upgraded per Teaming Annex A for voter-facing apps |
| UUID7 for all generated IDs | Monotonic + time-ordered + random; consistent with sw/ back-end standard |
| Pydantic v2 for any back-end model surface touched by mobile APIs | Consistent with sw/ Python async back-end standard |

---

## 5. Open Questions for the Stitching Pass

The following items must be resolved when merging these modules into the main sw/ specification documents. None of the protected files were touched; these are flags for the stitching agent.

### 5.1 Module index in sw/PRD.md

The sw/PRD.md v2 (currently being written by a parallel agent) must include M29-M34 in its module index table and in the "Mobile Subsystem" subsection. Add a reference to `sw/mobile/PRD.md` for the full mobile PRD.

### 5.2 sw/ARCHITECTURE.md service-topology addition

The sw/ARCHITECTURE.md must add a "Mobile App Layer" node to the service topology diagram showing the six apps connecting to the IEBC Edge layer via mTLS. Add a reference to `sw/mobile/ARCHITECTURE.md`.

### 5.3 sw/DATA-MODEL.md entity additions

The following new entities or extensions are implied by M29-M34 and must be added to the data model:
- `AgentCountersignature` — party-agent HSM-backed signature on Form 34A (M30)
- `ObserverVisitLog` — GPS check-in + check-out records per observer per station (M34)
- `PersonnelAttendanceRecord` — check-in/check-out with biometric attestation (M33)
- `MobileIncidentReport` — generalised incident report with attachment blobs, from any of M29-M34; may extend existing `IncidentReport` entity if one exists
- `FieldSupportTicket` — engineer-side ticket record (M32); may already exist in O02 ticketing schema

### 5.4 sw/SECURITY.md mobile section

The sw/SECURITY.md must add a "Mobile Security" section referencing `sw/mobile/SECURITY.md` and the SR-M-* requirement namespace. The SR-M-* requirements are not yet in the main security spec.

### 5.5 sw/ACCEPTANCE-CRITERIA.md mobile gates

The sw/ACCEPTANCE-CRITERIA.md must add a "Mobile Suite" gate section pointing to `sw/mobile/ACCEPTANCE-CRITERIA.md` and the per-module criteria in M29-M34 §8.

### 5.6 sw/API-CATALOG.md mobile endpoints

The following API endpoints are consumed by M29-M34 but may not yet be documented in the API catalog:
- Read-only BVI dashboard endpoint for M31 companion app
- Agent countersignature submission endpoint for M30 (likely in M10 scope)
- Observer visit log submission and sampling endpoint for M34 (likely new)
- Personnel attendance check-in / check-out endpoints for M33 (likely in O05 scope)
- Field support ticket queue API for M32 (likely in O02 scope)
- Spares availability query API for M32 (in M20 scope)

### 5.7 sw/DATACRAFT_SOFTWARE_SCOPE.md subsystem enumeration

The DATACRAFT_SOFTWARE_SCOPE.md v2 (currently being written by a parallel agent) must add M29-M34 to the software scope enumeration and cross-reference the cost/effort estimates. The Personnel Dispatch back-end (O05, extended from MeGuard) should be confirmed as in-scope there.

### 5.8 O05 Personnel Dispatch Backend specification

The O05 Personnel Dispatch back-end is referenced by M33 but may not have its own module spec. The stitching pass should check whether O05 is specified and, if not, create it or confirm it is covered by the MeGuard integration notes.

### 5.9 Module renumbering risk

Modules M13-M28 were being written by parallel agents during this session. If any of those agents used M29+ numbers for something other than mobile apps, a renumbering conflict will exist. The stitching pass must check all module files for numbering collisions before publishing the merged index.

---

## 6. Source-of-Truth Status

The following source files in `mobile/` remain the canonical specification pending stitching-pass consolidation. They were NOT modified during this integration.

| Source file | Status |
|---|---|
| `mobile/README.md` | SOURCE OF TRUTH — not modified |
| `mobile/PRD.md` | SOURCE OF TRUTH — not modified |
| `mobile/ARCHITECTURE.md` | SOURCE OF TRUTH — not modified |
| `mobile/SECURITY.md` | SOURCE OF TRUTH — not modified |
| `mobile/ACCEPTANCE-CRITERIA.md` | SOURCE OF TRUTH — not modified |
| `mobile/apps/MA01-iebc-verify.md` | SOURCE OF TRUTH — not modified |
| `mobile/apps/MA02-agent.md` | SOURCE OF TRUTH — not modified |
| `mobile/apps/MA03-poll-clerk.md` | SOURCE OF TRUTH — not modified |
| `mobile/apps/MA04-field-support.md` | SOURCE OF TRUTH — not modified |
| `mobile/apps/MA05-personnel-dispatch.md` | SOURCE OF TRUTH — not modified |
| `mobile/apps/MA06-observer.md` | SOURCE OF TRUTH — not modified |

After the stitching pass, the sw/ versions (M29-M34 and sw/mobile/) become the canonical specification and the mobile/ folder may be archived or retained as historical.

---

## 7. Confirmation: Protected Files Not Touched

The following files were protected and were not touched during this integration:

- `sw/PRD.md` — not touched
- `sw/ARCHITECTURE.md` — not touched
- `sw/DATA-MODEL.md` — not touched
- `sw/SECURITY.md` — not touched
- `sw/API-CATALOG.md` — not touched
- `sw/ACCEPTANCE-CRITERIA.md` — not touched
- `sw/DATACRAFT_SOFTWARE_SCOPE.md` — not touched
- `sw/modules/M13-*.md` through `sw/modules/M28-*.md` — not touched (written by parallel agents)

---

## 8. Top-3 Recommendations for the Stitching Pass

**1. Extract the Rust FFI verifier as a shared library.** Both M13 (Public Verifier App) and M29 (IEBC Verify) depend on the same Merkle-inclusion-proof validation and HSM-signature verification logic. M34 (Observer) and M30 (Agent) use the same verifier for countersignature flows. Creating a single `libiebc-verifier` Rust crate referenced by all four apps prevents drift and reduces the attack surface for cryptographic code.

**2. Confirm O05 Personnel Dispatch Backend scope before contract signature.** M33 (IEBC Dispatch) is the largest by user volume (250K on peak day) and depends on a back-end that extends Datacraft's MeGuard pattern. If O05 is not yet fully scoped in DATACRAFT_SOFTWARE_SCOPE.md, the cost and effort estimate will be incomplete. This is the highest schedule risk in the mobile suite.

**3. Run a numbering collision check on M13-M28 before publishing the merged module index.** Three parallel agents wrote M13-M20, M21-M28, and DATACRAFT_SOFTWARE_SCOPE.md v2 concurrently with this agent. A simple `grep -r "M29\|M30\|M31\|M32\|M33\|M34" sw/modules/M1*.md sw/modules/M2*.md` will reveal any collisions before the index is published.
