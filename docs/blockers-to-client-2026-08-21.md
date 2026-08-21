# Blockers to client — 2026-08-21

Items in the Datacraft `sw/` response corpus that block our downstream work and require Datacraft or IEBC action to unblock. Raised as a batch after the RFP-alignment review pass.

These are internal spec inconsistencies (from Datacraft's own `sw/INTEGRATION-AUDIT.md` as well as this review) plus IEBC-side clarifications outstanding. They are NOT design questions we can resolve alone.

## Priority 1 — commercial / scope blockers

### 1.1 DATACRAFT_SOFTWARE_SCOPE.md covers only M01-M28

- **Impact**: cost model, staffing plan, and delivery commitment for M29-M69 (41 modules) has no basis in the current scope document. We cannot cost these 41 modules or commit to their delivery.
- **Ask**: Datacraft extend the scope document to M29-M69 with the same cost/effort/dependency structure applied to M01-M28.
- **Also flagged in**: `sw/INTEGRATION-AUDIT.md` Blocker 2.

### 1.2 API-CATALOG.md covers only M01-M28

- **Impact**: client SDK generation, Pact contract test authoring, and integration work for the 41 later modules is blocked. Our `packages/api-clients/` scaffold and CI generation cannot proceed for M29+.
- **Ask**: extend API-CATALOG.md to M29-M69 with the same OpenAPI 3.1 shape.

### 1.3 ACCEPTANCE-CRITERIA.md has no OAT/FAT criteria for M35-M69

- **Impact**: UAT/OAT sign-off for 51% of the module set has no criteria basis. Delivery cannot be signed off.
- **Ask**: extend to full module set before contract award or immediately after.
- **Also flagged in**: `sw/INTEGRATION-AUDIT.md` Blocker 3.

## Priority 2 — legal / retention blockers

### 2.1 Voter register retention contradiction (5y vs 15y)

- **Impact**: DATA-MODEL.md says "contract + 5 years" while Elections Act § and expect.md require 15 years. Choosing wrong triggers either statutory non-compliance or over-retention DPA exposure.
- **Ask**: IEBC counsel confirms authoritative retention window for voter register (M02, M35). Update DATA-MODEL.md.
- **Also flagged in**: `sw/INTEGRATION-AUDIT.md` Blocker 1.

### 2.2 Digital signature + QR standards for Form 34A (PRE_BID C13)

- **Impact**: cannot spec the M08 RTS device signature scheme without IEBC's chosen algorithm / key-length.
- **Ask**: IEBC answer PRE_BID C13 with a specific signature suite (recommended: Ed25519 or ECDSA P-384 aligned with ADR-0005 hybrid stack).

### 2.3 Log retention specifics (PRE_BID D13)

- **Impact**: our ADR-0015 amendment aligned to 90-day online per RFP but the "permanent archive" details need confirming (medium, format, per-tier retention policy, deletion/erasure interaction).
- **Ask**: IEBC clarifies PRE_BID D13.

### 2.4 ODPC registration timing (PRE_BID D10)

- **Impact**: bidder needs to know whether ODPC data-controller registration must be in place at bid, at award, or before commercial operation.
- **Ask**: IEBC answers PRE_BID D10.

### 2.5 Kenya data residency for production data (PRE_BID D9)

- **Impact**: if IEBC mandates full Kenya-only residency for production data (not just publication), no offshore support access is possible. Affects Tier 1-3 support desk model, remote-hire mobility, and any offshore build / test / staging.
- **Ask**: IEBC answers PRE_BID D9.

## Priority 3 — housekeeping (Datacraft spec-doc contradictions)

### 3.1 Kit language stale text

- Datacraft's own INTEGRATION-AUDIT says Kotlin+Compose is "resolved" but stale Flutter mentions remain in older ARCHITECTURE.md text. Meanwhile, RFP-binding ~14,100 legacy Android device reuse argues for Flutter. **Recommend**: revert to Flutter for kits (ADR-0002 + ADR-0023 already reflect this).
- **Ask**: sweep the spec to make it consistent with Flutter across kits + mobile.

### 3.2 HSM level stale text

- expect.md says FIPS 140-2 L3; ARCHITECTURE + SECURITY concur on 140-3 L3+.
- **Ask**: update expect.md line 37 to 140-3 L3+ (or delete if superseded).

### 3.3 CI in air-gapped path

- ARCHITECTURE §14.1 names GitHub Actions / GitLab CI. Contradicts air-gap mandate.
- **Ask**: sweep to on-prem CI (Gitea/Forgejo + Woodpecker per ADR-0006).

### 3.4 Cloudflare/Akamai vs sovereignty

- ARCHITECTURE §3.1 / §12 name Cloudflare/Akamai; §17.2 mandates Kenya residency.
- **Ask**: reconcile per CR-003 (drop Cloudflare/Akamai, adopt Kenyan-ISP peering).

### 3.5 Escrow

- RFP explicitly says escrow NOT required (tender.txt L9556, L11377). Datacraft's plan referenced monthly rolling escrow deposits.
- **Ask**: Datacraft confirm escrow is dropped from scope. Cost savings should flow to another line item.

### 3.6 Three-site vs two-site DR

- Datacraft plan referenced three sites (primary + mirror + escrow). RFP mandates only two (primary + mirror, IEBC-provided).
- **Ask**: Datacraft confirm two-site posture. Third site retained only as optional reproducibility-bundle publication mirror if budget-approved.

### 3.7 Authz engine "OPA (or Cedar)" ambiguity

- SECURITY §3.1 says "OPA (or Cedar)"; ARCHITECTURE §14.4 commits to Cedar.
- **Ask**: reconcile per CR-005 (adopt SpiceDB for domain authz; retain OPA for admission).

## Priority 4 — informational / defensive posture

### 4.1 Kenya-based Tier 1-3 support desk model

- RFP mandates Kenya-based support. Datacraft's staffing plan needs to reflect this.
- **Ask**: Datacraft publish the on-shore staffing plan; validate our on-call rota assumptions in ADR-0022.

### 4.2 Independent security certifier (PRE_BID D11)

- Elections (Technology) Regulations 2017 require an independent security certifier. IEBC has not named it.
- **Ask**: IEBC answers PRE_BID D11. Certifier's requirements may add compliance obligations we should bake into ADR-0009 / ADR-0022.

### 4.3 Bidder-provided vs IEBC-provided DC hardware (PRE_BID C1)

- If IEBC provides all DC hardware, our K8s cluster provisioning story is entirely on IEBC-owned tin. Deployment automation and hardware-selector work depends on this.
- **Ask**: IEBC clarifies PRE_BID C1.

---

Owner (Datacraft-side): TBD.
Owner (IEBC-side): TBD.
Next sync: TBD.
