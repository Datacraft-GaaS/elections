# Independent Assurance Board (IAB) Charter
# G05 — CLOSED (full functional governance document)
# M5 auditId=g05-iab-charter, proofHash=sha256(iab-charter-v2), signature=Ed25519-HSM
# ADR refs: 0004 (audit), 0010 (audit chain), 0022 (supply-chain integrity), 0023 (security)

## 1. Mandate (Part III PB-01 + PM-08)
The IAB provides independent verification of election integrity, audit chain
validity, and procedural compliance across all 20 EEL services.

## 2. Composition (5 members)
- 2 appointed by IEBC Chair (independent from management)
- 2 nominated by registered political parties (opposition representation)
- 1 nominated by civil society (Kenya Human Rights Commission / NCHR)

## 3. Authority
- Read-only access to `platform/observability/`, `detection/`, `chaos/scenarios/`
- Right to call emergency audit halt (Section 44A complementary mechanism)
- Signing authority on M5 audit chain amendments (requires 3-of-5 signatures)
- Direct reporting line to IEBC Chair + Attorney General (legal privilege)

## 4. Operations
- Pre-election: verify training-svc LMS audit logs + inventory RFIDs + BVI gallery
- Election-day: real-time access to portfolio (sanitised) via portal observer-view
- Post-election: verify transparency log + VRF sortition + court-admissible bundles

## 5. Term
- Appointment: 180 days before election day (T-180)
- Term ends: 90 days after final court determination (T+90)
- Quorum: 4 of 5 for decisions; 3 of 5 for audit sign-off

## 6. Audit Trail
Every IAB action emits M5 event (auditId, proofHash, signature, correlation_key).
