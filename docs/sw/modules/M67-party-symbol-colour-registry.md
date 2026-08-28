# M67 — Party Symbol and Colour Registry

**Module ID:** M67
**Name:** Party Symbol and Colour Registry
**Owner:** Datacraft Limited — Civic Participation Team
**Status:** Draft v1
**IP Classification:** Custom Software — IPR vests in IEBC per GCC 15.4

**Strategy traceback:** Political Parties Act 2011 §§7-9; upstream of M45 Party Registration and M12 CRMS ballot design

---

## Purpose

Trademark-style registry for political party visual identity: symbol (graphical mark), colour (Pantone + RGB + CMYK), and logotype. Upstream of M45 (Party Registration workflow) and M12 (CRMS ballot design). Enforces similarity checks (Tanimoto coefficient for shape, perceptual hash for image) before a new symbol is approved. Maintains colour exclusivity within election cycle. Manages rebranding workflow with mandatory differentiation test. Provides ballot-design export API consumed by M12. Political Parties Act 2011 §§7-9.

---

## User Personas

| Persona | Access | Primary task |
|---|---|---|
| Party Registrar (applicant) | Write (own party) | Submit symbol, colours, logotype; file rebrand |
| IEBC Registrar Officer | Write (all) | Review submissions, approve/reject, manage conflicts |
| M12 CRMS System | API consumer | Fetch approved symbol and colour for ballot design |
| M45 Party Registration | API consumer | Trigger symbol registration at party-registration step |
| IAB Member | Read-only | Audit registry and conflict resolutions |
| Public | Read-only (public endpoint) | View registered party symbols |
| Legal Team | Read-only + export | Produce registry certificates |

---

## Functional Requirements

### FR-67.01 — Symbol submission
Party submits: SVG vector file (preferred, max 5 MB), fallback PNG (min 300 dpi, max 10 MB), description text (max 200 chars). System generates perceptual hash (pHash, 64-bit) and Tanimoto similarity score against every registered symbol. Submission also stores normalised bitmap at 512x512 for comparison.

### FR-67.02 — Similarity conflict detection
On submission: compute pHash distance against all currently registered symbols. Flag conflict if pHash Hamming distance < 10 (high similarity threshold; configurable). Secondary check: Tanimoto coefficient on pixel-presence bitmask. Any conflict above threshold: auto-reject with specific conflicting symbol identified. Registrar Officer may override with written determination.

### FR-67.03 — Colour registration
Party registers primary colour (max 3: primary, secondary, accent). Each colour specified as: Pantone PMS code, RGB (0-255), CMYK percentage, RAL code (optional). M67 maps Pantone to RGB via ICC profile database. Uniqueness check: CIE76 delta-E < 10 between registered primary colours triggers conflict warning. Registrar Officer decides.

### FR-67.04 — Logotype registration
Party registers wordmark/logotype: SVG or high-res PNG. Stored separately from symbol. M12 consumes logotype for ballot-paper header design. Logotype does not trigger similarity check (typefaces are not exclusive).

### FR-67.05 — Approval workflow
Registrar Officer reviews submission: symbol similarity report, colour conflict report, completeness check. Determination: `approved` / `rejected_similarity` / `rejected_incomplete` / `deferred_pending_info`. Approved records: stored with approval timestamp, officer UUID, hash chain entry. Rejected: rejection reason code + narrative; party notified via M45 communication channel.

### FR-67.06 — Rebrand workflow
Party requests rebrand: new symbol + colour submitted as draft. Old registration remains active until rebrand approved. Rebrand subject to same similarity check against all other parties. Rebrand approval: dual officer sign-off (Registrar Officer + Deputy Registrar). Approved rebrand: old record archived (immutable), new record activated. Ballot-design systems auto-notified via webhook.

### FR-67.07 — Ballot design export API
M12 CRMS and Al Ghurair ballot printer consume: per-party ballot design package including approved symbol (SVG), primary colour (CMYK for print, RGB for screen), logotype, party name. Package versioned; M12 requests specific version. Any mid-cycle rebrand approved after T-90 requires IEBC Commissioner sign-off.

### FR-67.08 — Registry certificate
On demand: signed PDF certificate per party listing registered symbol hash, colours, approval date, officer signature. Used for legal and media purposes.

### FR-67.09 — Public registry browser
Publicly accessible list of all registered parties with symbols and colour swatches. Download options: PNG (300 dpi) per symbol. Colour values shown as HEX and Pantone name. WCAG 2.2 AA; EN and SW.

### FR-67.10 — Election-cycle immutability
From T-90: no new symbol registrations or rebrands permitted without Commissioner sign-off. Registry snapshot for the current cycle is immutable from T-90. M12 receives final snapshot at T-90 via signed manifest.

---

## Non-Functional Requirements

### NFR-67.01 — WCAG 2.2 AA
Public registry browser and party submission portal meet WCAG 2.2 AA. Colour contrast checks: all registered party colours tested against white and black backgrounds for WCAG AA ratio (4.5:1). Accessibility report published with registry.

### NFR-67.02 — Similarity computation
Perceptual hash and Tanimoto comparison: < 2 seconds per submission against a registry of 200+ parties.

### NFR-67.03 — Availability
99.9% during active registration period. Read API for M12/M45: 99.99% (cached CDN).

### NFR-67.04 — Integrity
All registry records append-only, hash-chained. No symbol or colour record can be modified; only new draft/approved records created. Every approval event signed by officer JWT.

### NFR-67.05 — DPA 2019
Party registrar contact details are personal data; stored separately from the symbol registry. Symbol and colour data are not personal data. Access to party registrar contact details scoped to IEBC staff only.

### NFR-67.06 — Print fidelity
CMYK values validated against colour-profile lookup table; warning if submitted RGB-to-CMYK conversion deviates > 5 delta-E from Pantone ICC profile. M12 receives CMYK values for physical ballot printing.

---

- **NFR-M67-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `iebc-iems-M67`; alert ruleset ID `iems-M67-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## User Stories

1. As a Party Registrar, I want to submit my party's symbol as an SVG and receive an instant similarity report, so that I know before formal submission whether a conflict exists.
2. As a Party Registrar, I want to register my party's Pantone primary colour and receive a conflict check against other registered parties, so that my ballot paper colour is distinctive.
3. As an IEBC Registrar Officer, I want to see the pHash similarity score and the conflicting symbol side by side, so that I can make a defensible determination on the rejection.
4. As an M12 CRMS system, I want to fetch the approved ballot design package for a given party via an authenticated API, so that ballot papers are designed with verified, current symbols.
5. As a party filing a rebrand, I want to submit new assets and have the old registration remain active until my rebrand is approved, so that existing ballot materials are not invalidated mid-process.
6. As a Legal Officer, I want to generate a signed PDF registry certificate for any party, so that the registration can be cited in legal proceedings.
7. As a member of the public, I want to browse all registered party symbols and colour codes on a public page, so that I can identify parties visually before election day.
8. As an IAB Member, I want to audit the conflict-detection log for any override decisions, so that I can flag cases where a Registrar Officer approved despite a high similarity score.
9. As the IEBC Commissioner, I want to be notified and required to co-sign any rebrand request received within T-90, so that last-minute rebrands do not destabilise ballot printing.
10. As a visually impaired voter (assistive technology user), I want the public registry to have proper alt text on all symbol images and colour descriptions in text, so that the registry is fully accessible.
11. As a Ballot Printer (M12 consumer), I want to receive CMYK values validated against the party's submitted Pantone code, so that printed ballot colours are faithful to the registered identity.

---

## Data Model

```python
from pydantic import BaseModel, ConfigDict
from datetime import datetime

class PartyColour(BaseModel):
    model_config = ConfigDict(extra='forbid')
    role: str  # primary | secondary | accent
    pantone_pms: str
    rgb_r: int
    rgb_g: int
    rgb_b: int
    cmyk_c: float
    cmyk_m: float
    cmyk_y: float
    cmyk_k: float
    hex: str
    ral_code: str | None
    wcag_on_white: float  # contrast ratio
    wcag_on_black: float

class SymbolRegistration(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    party_id: str  # from M45
    cycle: str  # "2027_general"
    status: str  # draft | under_review | approved | rejected | archived
    svg_url: str
    png_url: str
    description: str
    phash: str  # 64-bit perceptual hash hex
    tanimoto_conflicts: list[dict]  # [{party_id, score}]
    colours: list[PartyColour]
    logotype_url: str | None
    submitted_at: datetime
    submitted_by: str
    reviewed_by: str | None
    reviewed_at: datetime | None
    determination: str | None
    determination_reason: str | None
    version: int

class BallotDesignPackage(BaseModel):
    model_config = ConfigDict(extra='forbid')
    party_id: str
    party_name: str
    symbol_svg_url: str
    logotype_url: str | None
    primary_colour: PartyColour
    package_version: int
    approved_at: datetime
    cycle: str
    signed_manifest_hash: str
```

---

## API (OpenAPI 3.1)

```yaml
openapi: "3.1.0"
info:
  title: M67 Party Symbol and Colour Registry API
  version: "1.0.0"
paths:
  /api/v1/party-registry/symbols:
    post:
      summary: Submit symbol registration
      security: [{BearerAuth: []}]
      requestBody:
        required: true
        content:
          multipart/form-data:
            schema:
              type: object
              properties:
                party_id: {type: string}
                svg_file: {type: string, format: binary}
                description: {type: string}
      responses:
        "202": {description: Submission accepted; similarity report included}
  /api/v1/party-registry/symbols/{id}:
    get:
      summary: Get symbol registration status
      security: [{BearerAuth: []}]
      responses:
        "200": {description: SymbolRegistration}
    patch:
      summary: Record determination (Registrar Officer)
      security: [{BearerAuth: []}]
      responses:
        "200": {description: Determination recorded}
  /api/v1/party-registry/ballot-package/{party_id}:
    get:
      summary: Get ballot design package (M12, M45, ballot printer)
      security: [{BearerAuth: []}]
      parameters:
        - name: party_id
          in: path
          required: true
          schema: {type: string}
        - name: cycle
          in: query
          required: true
          schema: {type: string}
      responses:
        "200": {description: BallotDesignPackage}
  /api/v1/party-registry/public:
    get:
      summary: Public registry browser
      parameters:
        - name: cycle
          in: query
          schema: {type: string}
      responses:
        "200": {description: List of approved registrations with public fields}
  /api/v1/party-registry/certificates/{party_id}:
    get:
      summary: Signed PDF registry certificate
      security: [{BearerAuth: []}]
      responses:
        "200":
          description: Signed PDF
          content:
            application/pdf:
              schema: {type: string, format: binary}
```

---

## Integrations

**Upstream:**
- M45 (Party Registration) — party_id source; triggers symbol submission step
- Party SVG/image upload (multipart)
- Pantone ICC profile database (licensed; offline)

**Downstream:**
- M12 (CRMS) — ballot design package API consumer
- Al Ghurair ballot printer — CMYK values for physical print
- M15 (IAB Dashboard) — registry audit log
- M11 (Public Portal) — public registry browser
- P05 (Audit log), P06 (HSM — certificate signing)

---

## UI/UX — Key Screens

1. **Symbol submission wizard** (party portal) — upload SVG/PNG, preview, similarity check report (instant), colour picker with Pantone search, logotype upload; WCAG 2.2 AA; EN/SW
2. **Registrar review queue** — submission list; side-by-side symbol comparison; pHash score; colour delta-E; approve/reject/defer buttons
3. **Conflict resolution screen** — overlaid symbol images; similarity metrics; override reason entry; dual-sign for borderline cases
4. **Public registry browser** — grid of party symbols with colour swatches; search by party name; download PNG; accessibility: alt text and colour-name text on every swatch
5. **Ballot design package preview** (M12 integration) — rendered ballot paper mockup with the party's approved symbol and colour

---

## Compliance Mapping

| Requirement | Source |
|---|---|
| Symbol registration requirement | Political Parties Act 2011 §7 |
| Colour and logotype registration | Political Parties Act 2011 §8 |
| Conflict adjudication process | Political Parties Act 2011 §9 |
| Ballot paper symbol requirement | Elections Act §44(2) |
| WCAG 2.2 AA accessibility | Strategy UI baseline |
| Print-fidelity CMYK | Al Ghurair ballot printing specification |

---

## Success Metrics and SLA

| Metric | Target | Sign-off |
|---|---|---|
| Symbol similarity report turnaround | < 2 seconds | Datacraft SRE |
| All parties registered before T-90 snapshot | 100% | IEBC Registrar |
| Conflict resolutions with written determination | 100% | IEBC Legal |
| Ballot design packages delivered to M12 before T-90 | 100% | IEBC ICT |
| WCAG 2.2 AA audit pass | 100% on public registry | Accessibility auditor |
| Registry certificate generation | < 30 seconds | Datacraft SRE |

---

## Test Strategy

- **Unit:** pHash computation, Tanimoto similarity, CIE76 delta-E colour distance, CMYK validation against Pantone ICC
- **Integration:** M45 party-id trigger, M12 ballot-package consumer, HSM certificate signing
- **Similarity:** Test suite of 200 symbol pairs with known similarity ground truth; detection rate >= 99%
- **Accessibility:** Automated axe-core + manual screen-reader test on public registry and submission wizard
- **Print fidelity:** CMYK output for 5 test colours compared against Pantone colour bridge; delta-E < 5 required
- **Dry run T-90:** All registered parties fetch ballot packages; M12 renders mock ballot papers

---

## Rollout

| Phase | Date | Milestone |
|---|---|---|
| Pantone ICC profile database licensed and loaded | 2026-12-01 | Colour validation ready |
| Symbol submission portal live | 2027-01-01 | Open to parties |
| M45 integration for symbol-step trigger | 2027-01-15 | Linked to party registration |
| M12 ballot-package API consumed in staging | 2027-04-01 | Ballot design test |
| Registry snapshot locked (T-90) | 2027-05-11 | No further changes without Commissioner |
| Final ballot packages to M12 and printer | 2027-05-15 (T-86) | Ballot printing starts |

---

## Dependencies

- **M45** (Party Registration) — party_id and registration trigger
- **M12** (CRMS) — ballot design consumer
- **M15** (IAB Dashboard), **P05** (Audit log), **P06** (HSM)
- External: Pantone ICC profile licence, Al Ghurair print spec

---

## Risks and Mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | Party submits low-resolution bitmap instead of SVG | System auto-rejects below 300 dpi; SVG strongly preferred; submission guidance in portal with example files |
| R2 | Two parties dispute colour similarity near threshold | Registrar Officer written determination required; IAB can review; colour-difference displayed numerically to both parties |
| R3 | Rebrand request approved too close to ballot printing | T-90 hard freeze; any post-T-90 rebrand requires Commissioner sign-off and triggers re-print cost assessment |
| R4 | Symbol similarity engine misses near-identical marks due to rotation or scaling | Pre-processing: normalize to 512x512, 4-angle rotation test, scale-invariant features; tested on known-similar pairs |
| R5 | M12 consumes stale ballot package after late rebrand | Versioned package API; M12 always fetches by cycle + latest-approved version; webhook on any new approval triggers M12 refresh |

---

## Cross-References

- **M45** Party Registration — triggers symbol submission
- **M12** CRMS — ballot design consumer
- **M15** IAB Dashboard — registry audit
- **M11** Public Portal — public registry

---

## Strategy Traceability

| Section | Requirement |
|---|---|
| Political Parties Act 2011 §7 | Symbol registration |
| Political Parties Act 2011 §8 | Colour and logotype registration |
| Political Parties Act 2011 §9 | Conflict adjudication |
| Elections Act §44(2) | Ballot paper symbol requirement |

## 18. Decommissioning

**Trigger:** End of the 3-year IEMS framework (approximately 2030-08), or earlier if IEBC terminates the contract.

**Steps:**

1. **Data disposition** — all personal data purged or transferred to M69 (Long-Term Archive) per the retention schedule in DATA-MODEL.md §11 and DPA 2019 s.41. Disposal certificate signed by Datacraft DPO and IEBC DPO.
2. **Service shutdown** — Kubernetes deployments scaled to zero; PersistentVolumeClaims retained for 30 days then deleted after confirmation of M69 transfer.
3. **Credential revocation** — service account certificates, API keys, and HSM key handles revoked via P06 key-destruction ceremony within 48 hours of shutdown.
4. **Source-code escrow** — final tagged release deposited to C04 escrow within 7 days of shutdown.
5. **Runbook handover** — operational runbook and post-mortem notes delivered to IEBC ICT within 14 days.

**Owner:** SRE Lead. **Sign-off:** IEBC ICT Director + Datacraft Programme Director.

> **RESOLUTION (F-23, 2026-08-21):** Decommissioning stub added per audit finding. Owner: SRE Lead. Target: FAT-07 gate.
