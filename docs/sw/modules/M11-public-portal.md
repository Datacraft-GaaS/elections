# M11 — Public Results Portal

**Runs on:** CDN edge (static-first), origin in primary DC (update generator only)
**Owner:** Datacraft
**IPR:** Custom Software vests in the Commission per GCC 15.4

## 1. Purpose

Publish every scanned Form 34A / 34B / 34C / 35 / 36 / 37 / 38 with cryptographic provenance so that any Kenyan citizen can independently verify the result of their own polling station. Serve the results-day flash crowd — potentially 5M unique visitors in the first hour — without letting DDoS on origin disrupt public verification.

## 2. Personas

- Public (voters, media, civil society, observers, foreign observers)
- Party HQ analysts
- Auditors
- Petitioners retrieving evidence

## 3. Functional Requirements

- **FR-M11-001** Home dashboard: national totals per elective position + turnout with time-of-last-update.
- **FR-M11-002** Drill-down by county → constituency → ward → polling station.
- **FR-M11-003** Per-polling-station page: signed Form 34A image, extracted tally per candidate, Merkle-inclusion proof, transparency-log index, PO + agent signatures.
- **FR-M11-004** Bulk download of signed forms per constituency / county / national with digital signature bundles.
- **FR-M11-005** Transparency-log root retrieval: any citizen can fetch the current root and any historical root.
- **FR-M11-006** Inclusion-proof retrieval: given a Form 34A ID or a polling station code + election, return the Merkle-inclusion proof.
- **FR-M11-007** Static-first architecture: pages served from CDN cache; origin only serves the update generator that signs new snapshots.
- **FR-M11-008** Signed URLs on CDN so tampered snapshots are detectable.
- **FR-M11-009** Public transparency dashboard: every published root, every quorum operation, every high-severity incident (anonymised).
- **FR-M11-010** Multilingual: English + Kiswahili; extensible.
- **FR-M11-011** Accessibility: WCAG 2.2 AA; screen reader; keyboard navigation; text-only fallback for slow connections.
- **FR-M11-012** Mobile-first responsive design.
- **FR-M11-013** RSS / Atom feed of published forms for automation-friendly consumers.
- **FR-M11-014** OpenAPI JSON endpoints for programmatic access with rate limit.
- **FR-M11-015** No PII exposure: forms show only what statute requires; agent identities anonymised.
- **FR-M11-016** Provenance overlay on each form: "Verified by the Commission HSM signature · Merkle-included in transparency log at index N · Escrow-mirrored to [custodian]".
- **FR-M11-017** Live turnout feed with delayed / aggregated policy to prevent gaming.

## 4. Non-Functional Requirements

- **NFR-M11-001** 5M unique visitors in 1 hour peak; 50M total in first 24 h.
- **NFR-M11-002** Page load p99 ≤ 2 s at peak.
- **NFR-M11-003** Origin protected by upstream DDoS scrubbing + anycast + WAF + L7 challenge for anonymous read traffic during publication windows.
- **NFR-M11-004** 99.99% availability read; degrade to signed static snapshots on origin loss.
- **NFR-M11-005** Every response carries a signature verifiable against the Commission public key published on canonical domains.
- **NFR-M11-006** Active-active HA: M11 is a hub module with 18+ downstream dependents (M13, M14, M15, M21, M22, M26, M34, M41, M44, M49, M51, M63, M66, M68). CDN edge serves read traffic; origin is active-active across primary and mirror DCs. Active-passive is insufficient given downstream dependency count. Verified at FAT.

## 5. Integrations

- RTS Back-End (M10) for publication events
- BVI Back-End (M06) for live turnout feed
- Transparency log service
- Escrow mirror custodian
- CDN (Cloudflare / Akamai class)

## 6. Failure Modes

| Failure | Response |
|---|---|
| DDoS on origin | Static snapshots on CDN continue to serve; origin can be taken offline without user impact |
| Signature verification failure | Alert; page marked as unverifiable |
| CDN cache poisoning attempt | Signed URLs detect; requests re-fetched |
| Transparency log unreachable | Cache last N roots; degrade to cache; alert |
| Petition-heavy download load | Bulk downloads throttled; scheduled off-peak windows offered |

## 7. Acceptance Criteria

| # | Criterion | Method |
|---|---|---|
| A-M11-01 | 5M unique visitors/hour sustained | Load test |
| A-M11-02 | DDoS on origin does not stop portal | Chaos exercise |
| A-M11-03 | Merkle-inclusion proof retrievable ≤ 30 s per request | Bench |
| A-M11-04 | WCAG 2.2 AA audit pass | Independent audit |
| A-M11-05 | Signed snapshot verifiable via Commission public key | Independent verification |
| A-M11-06 | No PII exposure in any published surface | Manual + automated scan |
| A-M11-07 | Multilingual switch works EN / SW | UAT |

## 8. Cross-References

- `M10-rts-backend.md`
- `../SECURITY.md` §3.10
- `../THREAT_MODEL.md` §12–13
