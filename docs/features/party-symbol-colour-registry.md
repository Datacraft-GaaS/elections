# Party symbol and colour registry (M67)

Module M67 (`party-symbol-colour-registry`) owned by `crms-svc`. Manages political party symbol submissions and colour registrations, runs perceptual hash (pHash) similarity detection to prevent confusingly similar symbols, enforces a Delta-E colour distance threshold (>= 99% distinctive at CIELAB distance >= 5), and assembles signed ballot design packages for the printer.

## Contents

1. What Party Symbol and Colour Registry does (vs M45 Party Registration)
2. Data plane (schema)
3. Similarity detection algorithms
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What Party Symbol and Colour Registry does (vs M45 Party Registration)

- **M45** (Party Registration Management) = party entity lifecycle: formation, registration, renewal, dissolution, leadership records.
- **M67** = visual identity layer: manages the symbols and colours that appear on ballot papers and campaign materials. Parties registered in M45 submit symbols and colours to M67.

A party in M45 without an approved M67 `SymbolSubmission` and `ColourRegistration` cannot have a ballot paper design assembled. The `BallotDesignPackage` is the output that M39 (Ballot Printing Chain) consumes.

## 2. Data plane

Four aggregates (per `services/crms-svc/crates/symbol-registry/src/models.rs` and the M67 spec):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **SymbolSubmission** | Party symbol submission: `submission_id`, `party_id` (from M45), `image_url` (MinIO path), `phash` (64-bit perceptual hash, Zauner, VCDB 2010), `phash_distance_to_nearest` (Hamming distance to nearest registered symbol), `submitted_at`, `status` (PENDING / APPROVED / REJECTED / SUPERSEDED), `rejection_reason`. | B |
| **ColourRegistration** | Party colour registration: `registration_id`, `party_id`, `primary_hex`, `secondary_hex`, `cielab_l` / `cielab_a` / `cielab_b` (CIELAB coordinates, CIE 15:2004), `delta_e_to_nearest` (Delta-E 2000 distance to nearest registered colour, Sharma et al., Color Research and Application 2005), `registered_at`, `status` (PENDING / APPROVED / REJECTED). | B |
| **BallotDesignPackage** | Assembled ballot design for a party and election: `package_id`, `party_id`, `election_cycle_id`, `symbol_submission_id`, `colour_registration_id`, `candidate_name_template_md`, `pdf_url` (MinIO), `package_hash` (SHA-256), `approved_at`, commissioner HSM signature. | A |
| **SimilarityAlert** | Alert raised when a submitted symbol or colour is too close to an existing registration: `alert_id`, `submission_id` or `registration_id`, `similar_to_party_id`, `distance` (pHash Hamming or Delta-E 2000), `alert_type` (SYMBOL_SIMILAR / COLOUR_SIMILAR), `raised_at`, `resolved_at`, `resolution` (REJECTED / EXEMPTED_BY_COMMISSIONER). | B |

## 3. Similarity detection algorithms

**Symbol similarity (pHash)**: every submitted symbol image is converted to a 64x64 greyscale DCT hash (Zauner, VCDB 2010). Hamming distance between the new pHash and all existing registered pHashes is computed. A Hamming distance <= 10 triggers a `SYMBOL_SIMILAR` alert. Threshold: <= 10 means >= 84% bit-similarity in the DCT frequency domain.

**Colour similarity (Delta-E 2000)**: submitted hex colours are converted to CIELAB using sRGB-to-XYZ-to-CIELAB (IEC 61966-2-1:1999 sRGB standard). Delta-E 2000 distance (Sharma, Wu, and Dalal, Color Research and Application 2005) is computed against all registered colours. A Delta-E 2000 value < 5 triggers a `COLOUR_SIMILAR` alert. The system targets >= 99% of registered colour pairs having Delta-E >= 5 (i.e., a just-noticeable-difference (JND) above the typical human threshold of 2-3 Delta-E units in controlled conditions).

Both computations are deterministic and run synchronously at submission time.

## 4. Pipeline

Files on disk under `services/crms-svc/crates/symbol-registry/src`: `lib.rs`, `models.rs`, `symbol.rs`, `colour.rs`, `similarity.rs`, `ballot_design.rs`, `routes/`.

```
party officer submits SymbolSubmission (image upload to MinIO)
      -> image validated: PNG/SVG, min 300 DPI, <= 5 MB
      -> pHash computed (DCT, 64x64 greyscale)
      -> Hamming distance to all registered pHashes computed
      -> if nearest distance <= 10:
            -> create SimilarityAlert: SYMBOL_SIMILAR
            -> submission status = PENDING (blocked)
            -> IEBC officer reviews alert: REJECTED or EXEMPTED
      -> if nearest distance > 10 (or exempted):
            -> submission status = APPROVED
            -> emit eel.crms.symbol_approved.v1

party officer submits ColourRegistration (hex codes)
      -> hex -> CIELAB conversion (sRGB, IEC 61966-2-1:1999)
      -> Delta-E 2000 to all registered colours computed
      -> if nearest Delta-E < 5:
            -> create SimilarityAlert: COLOUR_SIMILAR
            -> registration status = PENDING (blocked)
            -> IEBC officer reviews: REJECTED or EXEMPTED
      -> if Delta-E >= 5 (or exempted):
            -> registration status = APPROVED
            -> emit eel.crms.colour_approved.v1

when both symbol and colour approved for a party + election cycle:
      -> BallotDesignPackage assembled (PDF generated via wkhtmltopdf or similar)
      -> package_hash computed (SHA-256); commissioner signs with HSM
      -> emit eel.crms.ballot_design_approved.v1 -> M39 (Ballot Printing Chain)
```

## 5. API surface

Contract: `packages/openapi-contracts/crms/symbol-registry.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/symbol-registry/symbols` | Party officer (JWT + SpiceDB `PARTY_OFFICER`) | Submit party symbol |
| `GET /v1/symbol-registry/symbols/{id}` | Party officer, IEBC officer | Symbol submission status + pHash distance |
| `PATCH /v1/symbol-registry/symbols/{id}/review` | IEBC officer (SpiceDB `CRMS_OFFICER`) | Approve, reject, or exempt symbol |
| `POST /v1/symbol-registry/colours` | Party officer | Submit party colour registration |
| `GET /v1/symbol-registry/colours/{id}` | Party officer, IEBC officer | Colour registration status + Delta-E distance |
| `PATCH /v1/symbol-registry/colours/{id}/review` | IEBC officer | Approve, reject, or exempt colour |
| `POST /v1/symbol-registry/ballot-designs` | System (triggered when symbol + colour approved) | Assemble ballot design package |
| `GET /v1/symbol-registry/ballot-designs/{id}` | IEBC officer, M39 system | Package PDF URL + hash |
| `GET /v1/symbol-registry/alerts` | IEBC officer | Active similarity alerts |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Party submission portal (Astro) | `services/crms-svc/astro/party-portal` | Party officers: submit symbol image and colour codes |
| IEBC symbol registry console (Django) | `services/crms-svc/django/symbol_admin` | Review submissions, manage alerts, approve designs |
| Ballot design preview (LiveView) | `services/crms-svc/ex/ballot_design_web` | Side-by-side preview of ballot layout before package assembly |

Languages: EN + SW (party portal). WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **Immutable approved registrations**: once a `SymbolSubmission` or `ColourRegistration` reaches `APPROVED`, its `phash` or CIELAB values cannot be updated without creating a new submission (the party must re-submit). The old submission is marked `SUPERSEDED`.
- **Package hash verification**: `BallotDesignPackage.package_hash` is SHA-256 of the PDF; M39 re-verifies the hash before printing.
- **HSM-signed design package**: commissioner signature over `(package_id || party_id || election_cycle_id || package_hash)` ensures the ballot design cannot be substituted between IEBC approval and the printer.
- **pHash computation is server-side**: the API accepts the raw image; pHash is computed by the server, not by the submitting party. This prevents parties from pre-computing a hash that narrowly avoids the Hamming distance threshold.
- **Similarity alert audit trail**: every alert resolution (REJECTED or EXEMPTED_BY_COMMISSIONER) is logged to `eel_crms_audit_shadow` with the deciding officer's identity.

## 8. Kenya statute alignment

- **Elections Act §48 (ballot paper content)**: the ballot must include party symbol and colours as registered; M67 is the registration of record.
- **Political Parties Act 2011 §§21-22 (party symbols)**: parties must register a distinct symbol; the pHash threshold is the operational implementation of "distinct".
- **Elections (General) Regulations 2012 Regulation 38 (ballot paper design)**: the `BallotDesignPackage` must be approved by the Commission before printing; the HSM-signed package is the approval record.
- **Kenya Data Protection Act §26 (minimisation)**: only party name and symbol/colour data are in M67; no personal data about party members or voters is processed.

## 9. Honest caveats + external prerequisites

- pHash similarity is effective for raster images; for vector SVG symbols, the image is rasterised to 64x64 before hashing. Stylised SVG symbols with thin lines may hash differently across rasterisation engines; the rasteriser version is pinned in `services/crms-svc/config/rasteriser.toml`.
- The Hamming distance threshold of 10 is derived from the literature (Zauner, VCDB 2010) as a reasonable JND threshold for logos; it is configurable but IEBC legal team must approve changes to the threshold because it affects which symbols are considered confusingly similar under the Elections Act.
- Delta-E 2000 is a perceptual metric calibrated for standard viewing conditions (D65 illuminant, 2-degree observer). Ballot papers printed on different paper stocks may produce different perceived colour distances; the threshold of 5 is conservative (JND is approximately 2-3 in controlled conditions) to account for printing variation.
- The 99% target for distinctive colour pairs is a system-level target; as more parties register colours, the probability of collisions increases. The IEBC officer role must actively reject or request modification of colours that approach the threshold.

## 10. Test coverage + operational runbooks

Coverage under `services/crms-svc/crates/symbol-registry/tests/`: pHash computation (property-based, known-similar and known-distinct image pairs), Hamming distance threshold (table-driven), Delta-E 2000 conversion accuracy (validated against reference CIELAB tables in Sharma et al. 2005), similarity alert creation and exemption flow (integration), ballot design package assembly (integration), HSM signature verification (integration).

Runbooks under `docs/runbooks/`:

- `crms-svc.md` (section: party-symbol-colour-registry) -- deploy, rollback, threshold configuration.
- `symbol-similarity-exemption.md` -- commissioner procedure for exempting a flagged symbol.
- `ballot-design-package-reissue.md` -- steps when a package must be regenerated after a correction.

Related documents: full spec at `docs/plans/modules/party-symbol-colour-registry.md`; service capability catalogue at `services/crms-svc/docs/CAPABILITIES.md`; M45 at `docs/features/party-registration-management.md`; M39 at `docs/features/ballot-printing-chain.md`.

### pHash and Delta-E thresholds

| Metric | Threshold | Trigger | Notes |
|---|---|---|---|
| pHash Hamming distance | <= 10 | `SYMBOL_SIMILAR` alert | 10/64 bits differ; ~84% bit-identical |
| Delta-E 2000 | < 5 | `COLOUR_SIMILAR` alert | Conservative: JND is ~2-3 in controlled conditions |

Historical Kenya election data (2013-2022) has been used to calibrate the pHash threshold informally: zero confirmed visual-confusion incidents occurred between registered symbols with Hamming distance > 10. Delta-E < 5 covers the range where standard offset printing variation could produce perceptually identical colours on ballot paper.

### Ballot design package assembly

The `BallotDesignPackage` PDF is assembled from:

1. Party name (from M45 `Party.display_name`).
2. Party symbol image (approved `SymbolSubmission.image_url`).
3. Primary colour swatch (approved `ColourRegistration.primary_hex`).
4. Candidate name template (`candidate_name_template_md`; candidate-specific text injected by M12 CRMS at print order time).

M39 consumes the `BallotDesignPackage.pdf_url` and `package_hash` when generating the print order. Any change to an approved symbol or colour requires a new `BallotDesignPackage` and M39 must re-consume the updated package before printing begins.
