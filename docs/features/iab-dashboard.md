# IAB dashboard (M15)

Module M15 (`iab-dashboard`) owned by `ecc-svc`. Independent Audit Board read-only console: tlog witness co-signing, L1-L4 escalation workflow, hardware-key (FIDO2/YubiKey) authentication, and quarterly audit report builder.

## Contents

1. What M15 does
2. Data plane
3. Ingest sources
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M15 does

M15 gives the Independent Audit Board (IAB) a read-only window into the system with two active capabilities: (1) co-signing the transparency log as an independent witness (if any witness diverges from the primary tlog, the system fails closed); (2) raising L1-L4 escalations against any finding. IAB members authenticate with hardware FIDO2 keys (YubiKey or equivalent). A quarterly report builder compiles IAB findings into a signed PDF submitted to the Commission and published.

The IAB cannot write to any election data store; all writes from M15 are confined to the escalation log and the tlog witness co-sign.

## 2. Data plane

Crates: `services/ecc-svc/crates/ecc-api/`, `services/transparency-svc/crates/tlog-witness/` (iab_client.rs).

| Aggregate | Purpose | Data class |
|---|---|---|
| **EscalationTicket** | ticket_id, severity (L1/L2/L3/L4), raised_by (IAB member hardware-key identity), subject_ref (tlog leaf, kit_serial, form_id, etc.), description, status (OPEN/ESCALATED/RESOLVED/DISMISSED), ts | B |
| **WitnessCoSign** | cosign_id, tlog_sth_hash, iab_member_key_id, cosig_bytes (Ed25519 via hardware key), ts | C |
| **IabAuditReport** | report_id, period_start, period_end, escalation_summary, findings_count, report_hash, iab_chair_sig, published_at | B |

## 3. Ingest sources

| Source | Notes |
|---|---|
| Transparency log (P05) | Read-only STH + leaf stream via `tlog-api/src/rest/` |
| tlog-witness (P05 witness protocol) | IAB is a registered witness; co-sign protocol per `tlog-witness/src/iab_client.rs` |
| M10 rts-backend IAB view | `rts-projections/src/iab_view.rs`; read-only tally summary |
| M02 bvr-backend | Read-only register snapshot access |
| M06 bvi-backend | Read-only turnout summary |
| ECC alert stream (M16) | SSE subscription to ECC alert feed; IAB sees all L3+ alerts |

## 4. Pipeline

Crates: `services/transparency-svc/crates/tlog-witness/src/iab_client.rs`, `services/ecc-svc/crates/ecc-api/src/`.

```
IAB member authenticates (FIDO2 hardware key: YubiKey / FIDO2 token)
      -> session established (JWT bound to hardware-key assertion)
      -> read-only console loaded:
             tlog STH stream (live, via tlog-api SSE)
             rts IAB view (read-only tally summary)
             turnout + register snapshots (read-only)

tlog witness co-sign cycle (configurable interval, default: per STH publish):
      tlog-witness/src/iab_client.rs:
      -> receive new STH from P05 tlog
      -> IAB member reviews STH consistency:
             does STH.tree_size > previous STH.tree_size? (monotone)
             does STH.root_hash pass consistency proof? (RFC 6962 s.2.1.3)
      -> IAB member approves -> hardware key signs STH hash (Ed25519)
      -> WitnessCoSign submitted to tlog-witness/src/publisher.rs
      -> tlog records IAB co-sign; quorum check (configurable N-of-M)
      -> if IAB refuses to co-sign: split_view alert (tlog-witness/src/split_view.rs)
             eel.transparency.witness_divergence.v1 emitted
             ECC alerted; system does NOT publish new STH until resolved

escalation workflow:
      IAB member POST /iab/v1/escalations (L1-L4)
      -> EscalationTicket created
      -> L3/L4: automatic ECC notification + Commissioner page
      -> Commission responds via internal channel; status updated
      -> EscalationTicket.status = RESOLVED / DISMISSED

quarterly report builder:
      IAB chair triggers report compilation
      -> aggregate EscalationTickets for period
      -> compile IabAuditReport (PDF via reports-templates)
      -> IAB chair signs with hardware key
      -> submitted to Commission; published to transparency-svc public Astro
```

## 5. API surface

Contract: `packages/openapi-contracts/iab/dashboard.yaml`.

| Method + path | Persona | Purpose |
|---|---|---|
| `GET /iab/v1/tlog/sth` | IAB member (FIDO2 JWT) | Current signed tree head |
| `GET /iab/v1/tlog/leaves` | IAB member (FIDO2 JWT) | Browse tlog leaves |
| `POST /iab/v1/tlog/cosign` | IAB member (FIDO2 JWT, hardware-key sig) | Submit STH co-sign |
| `GET /iab/v1/tally` | IAB member (FIDO2 JWT) | IAB view of tally summary (`iab_view.rs`) |
| `GET /iab/v1/turnout` | IAB member (FIDO2 JWT) | National turnout summary |
| `POST /iab/v1/escalations` | IAB member (FIDO2 JWT) | Raise L1-L4 escalation |
| `GET /iab/v1/escalations` | IAB member (FIDO2 JWT) | List escalation tickets |
| `POST /iab/v1/reports` | IAB chair (FIDO2 JWT, hardware-key sig) | Compile + sign quarterly report |

All endpoints are read-only except co-sign and escalation; no write access to election data stores.

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| IAB console (Elixir Phoenix / React) | `services/ecc-svc/ex/` or equivalent | Read-only dashboard + co-sign + escalation UI |
| Quarterly report viewer | `services/transparency-svc/astro/` | Published IAB reports (public) |

Languages: English (IAB is an international body; no Kiswahili requirement for this surface). WCAG 2.1 AA.

## 7. Integrity + anti-tamper

- **FIDO2 hardware-key authentication** -- no password fallback; FIDO2 assertion required for every session; hardware-key identity bound to WitnessCoSign.
- **Witness co-sign quorum** -- configurable N-of-M IAB members must co-sign each STH; single-member co-sign can be configured as minimum; quorum check in `tlog-witness/`.
- **Split-view detection** (`tlog-witness/src/split_view.rs`) -- if IAB witness sees a different tree than the primary tlog, `eel.transparency.witness_divergence.v1` is emitted and the system fails closed (no new STH published until resolved).
- **Consistency proof verification** (RFC 6962 s.2.1.3, Laurie, Langley, Kasper 2013) -- before co-sign, `iab_client.rs` verifies the consistency proof between the previous and current STH; monotonicity enforced.
- **Read-only access control** -- IAB SpiceDB role has no write permissions on any election data store; enforced at SpiceDB policy level.
- **Escalation audit trail** -- every EscalationTicket creation and status change logged to T1-audit with IAB member hardware-key identity.
- **Report signing** -- IabAuditReport hash signed by IAB chair hardware key before publication; signature included in published PDF.

## 8. Kenya statute alignment

- **Elections Act s.44 (transparency)** -- IAB witness role satisfies the statutory requirement for independent oversight of the electronic system.
- **Elections (Technology) Regulations 2017 Reg. 12 (audit)** -- IAB has access to all system logs and can verify the transparency log independently.
- **UNCHR / AU observer standards** -- FIDO2 hardware-key auth and read-only access model satisfies observer access requirements without granting write access.

## 9. Honest caveats + external prerequisites

- **IAB composition and mandate** are defined by statute and the Commission regulation, not this repo; M15 implements the technical access surface but does not define IAB membership or authority.
- **FIDO2 key provisioning** (YubiKey procurement, registration ceremony) is an operations prerequisite; M15 requires keys to be registered before election day.
- **Quarterly report SLA** -- M15 provides the report builder; IAB decides when to publish; no automated publishing without IAB chair signature.
- **Split-view response procedure** -- when `witness_divergence.v1` fires, resolution requires Commission and IAB agreement; `docs/runbooks/witness-divergence-response.md` documents the procedure (referenced from CVR doc, applies here too).
- **tlog-witness co-sign quorum** -- default N=1, M=3 (1 of 3 IAB members); the Commission may require higher quorum; configurable before deployment.

## 10. Test coverage + operational runbooks

| Layer | Harness | ID prefix |
|---|---|---|
| Unit (consistency proof verifier, co-sign quorum check, escalation FSM, report builder) | nextest | M15-UNIT- |
| Property (split-view always detected; co-sign quorum enforced at any N-of-M) | proptest | M15-PROP- |
| Contract (tlog-witness co-sign; ECC alert SSE; IAB tally view) | Pact | M15-PACT- |
| Integration (tlog double; IAB FIDO2 authenticator stub; ECC SSE) | nextest + testcontainers | M15-INT- |
| Security (FIDO2 bypass attempt; write-access escalation probe; split-view simulation) | Manual | M15-SEC- |

Runbook (to create): `docs/runbooks/iab-dashboard-degradation.md`. Related: `docs/runbooks/witness-divergence-response.md`.

Full spec: `docs/plans/modules/iab-dashboard.md`. Crates: `services/ecc-svc/crates/ecc-api/`, `services/transparency-svc/crates/tlog-witness/src/iab_client.rs`. IAB view: `services/rts-svc/crates/rts-projections/src/iab_view.rs`.
