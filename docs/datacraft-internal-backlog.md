# Datacraft internal backlog (bid documents)

Work on our own `work_docs/01-Projects/IEBC/sw/` documents, surfaced by `sw/INTEGRATION-AUDIT.md` and the ADR alignment review. These are not blockers on anyone else; they are ours to do. The `docs/plans/modules/*.md` pages in this repo are the source for the missing acceptance criteria and API coverage.

## Scope and coverage gaps

### 1.1 DATACRAFT_SOFTWARE_SCOPE.md covers only M01-M28
- **Impact**: cost model, staffing plan, and delivery commitment for M29-M69 (41 modules) has no basis in the current scope document. We cannot cost these 41 modules or commit to their delivery.
- **Action**: extend the scope document to M29-M69 with the same cost/effort/dependency structure applied to M01-M28.
- **Also flagged in**: `sw/INTEGRATION-AUDIT.md` Blocker 2.

### 1.2 API-CATALOG.md covers only M01-M28
- **Impact**: client SDK generation, Pact contract test authoring, and integration work for the 41 later modules is blocked. Our `packages/api-clients/` scaffold and CI generation cannot proceed for M29+.
- **Action**: extend API-CATALOG.md to M29-M69 with the same OpenAPI 3.1 shape.

### 1.3 ACCEPTANCE-CRITERIA.md has no OAT/FAT criteria for M35-M69
- **Impact**: UAT/OAT sign-off for 51% of the module set has no criteria basis. Delivery cannot be signed off.
- **Action**: extend to full module set before contract award or immediately after.
- **Also flagged in**: `sw/INTEGRATION-AUDIT.md` Blocker 3.

## Consistency edits to sw/

### 3.1 Kit language stale text
- Datacraft's own INTEGRATION-AUDIT says Kotlin+Compose is "resolved" but stale Flutter mentions remain in older ARCHITECTURE.md text. Meanwhile, RFP-binding ~14,100 legacy Android device reuse argues for Flutter. **Decision**: revert to Flutter for kits (ADR-0002 + ADR-0023 already reflect this).
- **Action**: sweep the spec to make it consistent with Flutter across kits + mobile.

### 3.2 HSM level stale text
- expect.md says FIPS 140-2 L3; ARCHITECTURE + SECURITY concur on 140-3 L3+.
- **Action**: update expect.md line 37 to 140-3 L3+ (or delete if superseded).

### 3.3 CI in air-gapped path
- ARCHITECTURE §14.1 names GitHub Actions / GitLab CI. Contradicts air-gap mandate.
- **Action**: sweep to on-prem CI (Gitea/Forgejo + Woodpecker per ADR-0006).

### 3.4 Cloudflare/Akamai vs sovereignty
- ARCHITECTURE §3.1 / §12 name Cloudflare/Akamai; §17.2 mandates Kenya residency.
- **Action**: reconcile per SA-003 (drop Cloudflare/Akamai, adopt Kenyan-ISP peering).

### 3.5 Escrow
- RFP explicitly says escrow NOT required (tender.txt L9556, L11377). Datacraft's plan referenced monthly rolling escrow deposits.
- **Action**: confirm escrow is dropped from scope. Cost savings should flow to another line item.

### 3.6 Three-site vs two-site DR
- Datacraft plan referenced three sites (primary + mirror + escrow). RFP mandates only two (primary + mirror, IEBC-provided).
- **Action**: confirm two-site posture. Third site retained only as optional reproducibility-bundle publication mirror if budget-approved.

### 3.7 Authz engine "OPA (or Cedar)" ambiguity
- SECURITY §3.1 says "OPA (or Cedar)"; ARCHITECTURE §14.4 commits to Cedar.
- **Action**: reconcile per SA-005 (adopt SpiceDB for domain authz; retain OPA for admission).

## Other

### 4.1 Kenya-based Tier 1-3 support desk model
- RFP mandates Kenya-based support. Datacraft's staffing plan needs to reflect this.
- **Action**: publish the on-shore staffing plan; validate our on-call rota assumptions in ADR-0022.

See also `docs/spec-alignment/` for the six stack-level edits.
