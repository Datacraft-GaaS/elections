# Grievance channels (M23)

Module M23 (`grievance-channels`) owned by `ecc-svc`. Provides six intake channels for public complaints and whistleblower tips: web form, SMS, USSD, WhatsApp, call centre, and walk-in. A SHA-256 hashed whistleblower identity profile protects source anonymity. SLA tiers P0/P1/P2 govern response time. Evidence attachments are transparency-log hashed. DCI/ODPP referrals are routed under EOA ss.6-12 with IEBC officer sign-off.

## Contents

1. What grievance channels does (vs related modules)
2. Data plane (schema)
3. Ingest sources
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What grievance channels does (vs related modules)

- **M22 Campaign finance transparency** = a specialised intake path for finance anomalies; M22 subscribes to M23 `FINANCE` category events.
- **M23** = generic public grievance intake across all categories (voter roll errors, polling station conduct, finance tips, intimidation, etc.); routes to the relevant module or escalates to regulators.
- **M11 Field support** = internal staff complaint channel; M23 is public-facing only.

M23 does not adjudicate grievances. Its output is a classified, triaged record with a reference number. Adjudication lives in the relevant domain service (M09, M22, M31, etc.). M23 tracks SLA compliance and escalates overdue tickets.

## 2. Data plane

Five aggregates under `services/ecc-svc/crates/ecc-api/src/models.rs`:

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **Grievance** | Public complaint: grievance_id (UUID7), channel, category (VOTER_ROLL / CONDUCT / FINANCE / OBSERVER / OTHER), description, status (OPEN/TRIAGED/ROUTED/CLOSED), sla_tier, received_at, closed_at. | B |
| **WhistleblowerProfile** | Anonymised source: profile_hash (SHA-256 of contact details), channel_ref, created_at. Never stores raw PII. | C |
| **EvidenceAttachment** | File or text proof: attachment_id, grievance_id, content_hash (P05 tlog ref), uploaded_at. | B |
| **SLAEvent** | SLA milestone: grievance_id, tier, event_type (BREACH_WARN / BREACH / ESCALATED), ts_hlc. | A |
| **RegulatoryReferral** | DCI/ODPP handoff: grievance_id, referral_body, assembled_by, chair_sig (HSM), sent_at. | A |

## 3. Ingest sources

| Source | Cadence | Format | Auth | Failure semantics |
|---|---|---|---|---|
| Web form | Real-time | JSON REST | reCAPTCHA v3 + rate-limit | Queue on overload; no loss |
| SMS (Africa's Talking / Safaricom) | Real-time | MO SMS webhook | Gateway HMAC | Retry; queue on gateway failure |
| USSD (`*400*1#`) | Real-time | USSD session | Gateway secret | Session timeout 90 s; partial submission saved |
| WhatsApp Cloud API | Real-time | Webhook JSON | Meta app secret | Queue on failure; 6-8 week approval prerequisite |
| Call centre agent | Real-time | Agent form | JWT (agent) | Agent submits on caller's behalf; caller reference number read back |
| Walk-in officer | On demand | Django form | JWT + SpiceDB INTAKE_OFFICER | Officer enters and confirms; grievance receipt printed |

## 4. Pipeline

```
intake (web / SMS / USSD / WhatsApp / call / walk-in)
  -> channel adapter normalises to Grievance schema
  -> if whistleblower flag set:
       WhistleblowerProfile created with SHA-256(contact_details);
       raw contact details NOT persisted
  -> category classifier (rule-based + M28 A3 LLM assist where confidence >0.85)
  -> SLA tier assigned:
       P0 (violence / intimidation): 4 h response SLA
       P1 (systematic conduct breach, finance ceiling): 24 h
       P2 (voter roll error, general complaint): 72 h
  -> evidence attachments: content_hash anchored in P05 tlog within 60 s
  -> reference number issued; SMS/USSD/WhatsApp reply sent to complainant
  -> routing:
       FINANCE category -> M22 Kafka event
       VOTER_ROLL -> M35 CVR officer triage
       CONDUCT -> ECC back-office queue
       OBSERVER -> IAB observer coordinator
  -> SLA timer started; breach warning at 80% of SLA window
  -> if P0 unacknowledged after 4 h:
       SLAEvent(ESCALATED) created; IEBC Commissioner notified
  -> DCI/ODPP referral:
       officer triggers RegulatoryReferral assembly
       IEBC Chair HSM signature required
       eel.ecc.referral_sent.v1 emitted (exactly-once)
       P05 Merkle anchor within 60 s
```

## 5. API surface

Contract: `packages/openapi-contracts/ecc/grievance.yaml`. All write endpoints enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /ecc/v1/grievances` | Public (reCAPTCHA) | Submit grievance |
| `GET /ecc/v1/grievances/{ref}` | Public (reference number) | Status check |
| `POST /ecc/v1/grievances/{id}/attachments` | Public (JWT-less, rate-limited) | Upload evidence |
| `GET /ecc/v1/grievances` | ECC officer (JWT + SpiceDB) | Triage queue with SLA countdown |
| `PATCH /ecc/v1/grievances/{id}` | ECC officer | Update status, add notes, route |
| `POST /ecc/v1/grievances/{id}/referral` | IEBC officer + Chair (JWT + HSM) | Assemble and send regulatory referral |
| `GET /ecc/v1/stats` | Public (CDN-cached, 15 min) | Aggregated grievance counts by category and status |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Public web form (Astro) | `services/ecc-svc/astro/ecc-public/` | Complaint submission; reference number display; status check; WCAG 2.2 AA; EN + SW + 3 regional |
| ECC back-office (Django) | `services/ecc-svc/django/ecc-admin/` | Officer triage queue; SLA countdowns; routing controls; referral assembly |
| Call centre agent form (Phoenix LiveView) | `services/ecc-svc/ex/ecc_call_centre/` | Real-time intake during call; caller hold script displayed |
| USSD/SMS status gateway | `integrations/ussd-gateway/ecc-adapter/` | Status replies routed to complainant reference number |

KSL video guidance available on the public web form for Deaf complainants. WCAG 2.2 AA on all human-facing surfaces.

## 7. Integrity + anti-tamper

- **SHA-256 whistleblower profile** -- raw contact details are hashed on ingestion and immediately discarded; the hash is stored for deduplication only; no pathway exists to reverse the hash; legal-privilege annotation on the `WhistleblowerProfile` row restricts database access to the ECC Compliance Officer only.
- **Evidence content hash in tlog** -- every attachment hash is anchored in the P05 transparency log within 60 s of upload; a court can verify the attachment has not been altered since submission without accessing the IEBC systems.
- **Exactly-once referral events** -- `eel.ecc.referral_sent.v1` uses the `eel-outbox-rs` transactional pattern; duplicate referral triggers are rejected using the idempotency key `grievance_id + referral_body`.
- **SLA audit trail** -- every SLA event (`BREACH_WARN`, `BREACH`, `ESCALATED`) is append-only; no officer can suppress a breach record.
- **Category classifier audit** -- M28 AI assist category decisions are logged with model_id, confidence, and prompt hash; human officer can override; override is logged separately.

## 8. Kenya statute alignment

- **EOA ss.6-12 (IEBC jurisdiction over election offences)** -- referral pipeline implements the statutory pathway; Chair HSM signature records IEBC's formal referral decision.
- **Elections Act s.79 (citizen right of complaint)** -- six intake channels satisfy the accessibility obligation; USSD and SMS serve low-bandwidth and feature phone users.
- **Whistleblower Protection Act 2022** -- SHA-256 profile and no-raw-PII design satisfy source anonymity; legal-privilege annotation satisfies the privilege protection obligation.
- **DPA 2019 s.26 (minimisation)** -- grievance description truncated to 2 000 characters; no additional personal data collected beyond what the complainant voluntarily provides; walk-in officer required to inform complainant of data use before submission.
- **POCAMLA 2009** -- finance category grievances routed to M22 where POCAMLA patterns are applied; M23 itself does not apply ML models to finance data.

## 9. Honest caveats + external prerequisites

- WhatsApp Cloud API approval requires a Meta business account and a 6-8 week approval process; the adapter is ready under `integrations/whatsapp/` and shares an approval with M22's whistleblower tip intake.
- Africa's Talking / Safaricom SMS gateway SLA is commercial; network congestion during election day can delay MO SMS delivery; the USSD channel is the recommended fallback for real-time intake.
- USSD `*400*1#` short-code allocation requires a Safaricom / CCK commercial agreement (I08 Tier-A integration, allocation pending).
- The M28 A3 LLM assist category classifier must be approved by the DPA 2019 automated-processing review before being enabled in production; until approval, rule-based classification is the sole path.
- The referral routing to DCI/ODPP assumes a known secure email or API endpoint for each body; these must be confirmed with the relevant agencies before the referral pipeline is activated.

## 10. Test coverage + operational runbooks

Coverage under `services/ecc-svc/crates/ecc-api/tests/`: unit tests for channel adapters (web, SMS, USSD, WhatsApp), category classifier, SLA timer, evidence hasher, referral builder; property-based tests (proptest) asserting SLA tier assignment monotonicity; contract tests (Pact) for M22 finance signal, M35 CVR routing, M28 AI assist; integration tests with Africa's Talking stub, Meta WhatsApp stub, SoftHSM referral signer, P05 tlog stub; e2e Playwright test covering web form submission through triage, routing, SLA breach, and referral assembly; load tests (k6) for 5 000 concurrent grievance submissions during election day peak.

Runbooks under `docs/runbooks/`:

- `ecc-degradation.md` (create): SMS gateway down; USSD session timeout spike; WhatsApp webhook failure; P0 SLA breach alert procedure.
- `regulatory-referral.md` (create): step-by-step for DCI/ODPP referral assembly and Chair HSM signing.

Dashboards: `platform/observability/ecc-svc/grafana-ecc.json` (grievance intake rate by channel, SLA breach rate by tier, referral count, category distribution). SLO: 99.9% intake success; P0 breach rate <1%; evidence hash anchor lag <90 s.

Related: `docs/plans/modules/grievance-channels.md`; `services/ecc-svc/docs/CAPABILITIES.md`; edge-case matrix EC-ECC-* in `docs/plans/08-edge-case-matrix.md`.
