# M14 — Reproducibility Bundle

**Module ID:** M14
**Name:** Reproducibility Bundle
**Owner:** Datacraft Limited — Platform Engineering Team
**Status:** Draft v2
**IP Classification:** Custom Software — IPR vests in IEBC per GCC 15.4; source released open-source (MIT licence) per Strategy §XI.6 Undertaking Letter

**Strategy traceback:** Part XI (all sections), Part IV §D, Part XII §XII.2 Wall 1, Part XIX §XIX.1 Layer 4, Part XV §XV.1 Tier C

---

## Purpose

The Reproducibility Bundle is the technical mechanism by which any independent observer with a standard laptop can re-derive every published election result — bit-identically — from published inputs alone, without trusting IEBC, Datacraft, or any other party. It closes every "hidden server" narrative that has plagued Kenyan elections since 2017 (Raila Odinga v IEBC [2017] eKLR para 272). The bundle packages: all signed Form 34A inputs, the aggregation pipeline in a Docker Compose environment, a `make reproduce` entry point, per-stage expected-output hashes, and vendored dependencies. Any mathematics graduate runs one command and gets a cryptographically verifiable match or mismatch against the IEBC-declared result. The module also manages hash publication to the Kenya Gazette, national radio bulletin, and five independent digital channels, and issues participation tokens to voters.

---

## User Personas

| Persona | Channel | Primary task |
|---|---|---|
| Independent researcher / graduate student | CLI (Docker) | Re-run full pipeline, compare output hashes |
| Data journalist | CLI + web | Download bundle, run reproduce, publish findings |
| Party agent / petitioner's technical advisor | CLI | Verify specific Form 34A inclusion in tally |
| IAB technical member | CLI + audit dashboard | Reproduce pipeline stages; commission second implementation |
| ELOG / Carter Center statistician | CLI | Cross-check national aggregation arithmetic |
| Supreme Court IT expert | CLI + affidavit | Expert-witness-ready reproducibility report |
| Voter (via M13 app) | Mobile | Participation token lookup |

---

## Functional Requirements

### FR-14.01 — Reproducible aggregation pipeline
The pipeline accepts signed Form 34A inputs and produces Form 34B (constituency) and Form 34C (national) outputs. All stages are pure functions: same inputs always produce bit-identical outputs. No wall-clock reads, no random seeds, no external state reads at runtime.

### FR-14.02 — make reproduce entry point
A single `make reproduce` command on any Linux/macOS machine with Docker installed: downloads the input manifest from the IEBC public endpoint (or uses a local cached copy), verifies the manifest hash against the published Merkle root, runs the pipeline in Docker Compose, computes SHA-256 of outputs, and prints MATCH or MISMATCH against published expected hashes.

### FR-14.03 — Bundle composition
Bundle contents: `docker-compose.yml`, `Makefile`, `inputs/manifest.json` (hash-verified), `pipeline/` (containerised stages with pinned OCI image digests), `expected/` (per-stage expected output hashes), `verify.sh` (independent hash comparison), `README.md`, `LICENSE` (MIT). Bundle compressed < 20 GB. All dependencies vendored (no network access during `make reproduce`).

### FR-14.04 — Input manifest
Manifest lists every Form 34A: station code, PDF URL, PDF SHA-256, JSON URL, JSON SHA-256, submission timestamp, kit serial, Merkle sequence number. Manifest itself has a SHA-256 that is published in the Gazette and cross-published to 5 channels.

### FR-14.05 — Per-stage expected hashes
After each pipeline stage (form ingestion, OCR validation, constituency aggregation, national aggregation), the expected output hash is written to `expected/stage-N.sha256`. Any deviation from expected is a discrepancy that triggers the bug-bounty award.

### FR-14.06 — Containerised pipeline with pinned images
Every stage is an OCI image with a pinned digest (`sha256:...`). Images are published to the IEBC container registry and mirrored on GitHub Container Registry. The Makefile pulls from both with failover.

### FR-14.07 — Merkle root publication schedule
Daily during E-30 through E+30: Merkle root published to (1) Kenya Gazette (via Government Printer API), (2) IEBC Twitter/X account, (3) GitHub Releases on `iebc/kiems-reproduce`, (4) IPFS (pinned node operated by Datacraft), (5) IAB's independent domain. All five must show the same root; any discrepancy triggers IAB alert.

### FR-14.08 — National radio bulletin
A structured plain-text root-hash announcement is delivered to KBC Radio at 19:30 daily during E-30 through E+30 for reading in the 20:00 news bulletin. Delivery via secure SFTP to KBC technical operations, confirmed with read receipt.

### FR-14.09 — Participation token issuance
At end of voting, each voter's presence in the station is represented by an anonymous participation token: a 256-bit random value generated on-device, stored in the kit's participation roll, transmitted to back-end, and readable (but not linkable to identity) via M13. The token is cryptographically unlinkable to the voter's biometric or ID; it is linkable only to "a voter authenticated at this station."

### FR-14.10 — Second-implementation validation
IAB commissions an independent second implementation of both the ABIS dedup engine and the aggregation pipeline. Both implementations must agree within published tolerance (zero at result-affecting level). Both are published open-source. Discrepancy triggers full manual review.

### FR-14.11 — Bug-bounty integration
First party to submit a verified bit-discrepancy between the reproduce-me bundle output and any published IEBC result wins a cash award (KES 500,000 for constituency-level; KES 2,000,000 for national-level discrepancy). Rules published on HackerOne. Datacraft funds the award pool.

### FR-14.12 — 15-year reproducibility commitment
Bundle is packaged against Ubuntu 26.04 LTS (supported through 2036). All dependencies pinned with SHA-256. Code escrow deposit includes the full bundle per §PB-04 commitment. Any observer should be able to reproduce election results until 2042 without internet access.

### FR-14.13 — Bundle publication timeline
Inputs manifest: published within 30 minutes of each Form 34A upload completing. Per-stage expected hashes: published within 60 minutes of each aggregation stage. Full bundle: published within 4 hours of Form 34C signing.

---

## Non-Functional Requirements

### NFR-14.01 — Compute requirement
`make reproduce` completes in <= 2 hours on a machine with 8-core CPU, 16 GB RAM, 100 GB free disk. Tested and certified against minimum spec before publication.

### NFR-14.02 — Bundle size
Compressed bundle < 20 GB. On 20 Mbps connection (Nairobi fibre baseline) download time < 3 hours.

### NFR-14.03 — Determinism
Pipeline is fully deterministic: bit-identical output on every run from identical inputs, regardless of execution environment, CPU architecture (x86-64 or ARM64 via emulation), or timestamp. Verified by automated multi-environment CI matrix.

### NFR-14.04 — Availability
Bundle download endpoint: 99.9% availability during E-7 through E+90. Hosted on 3 mirrors (IEBC CDN, GitHub Releases, IPFS). Any mirror sufficient.

### NFR-14.05 — Audit trail
Every bundle publication event, every Gazette submission, every radio bulletin delivery, and every bug-bounty claim is logged in the M05 audit log with HSM signature.

### NFR-14.06 — Privacy (DPA 2019)
Participation tokens are stored without linkage to voter identity. Input manifest contains station codes, form hashes, and tallies — no voter PII. DPIA confirms no sensitive personal data in any public bundle artefact.

---

## User Stories

1. As an independent researcher, I want to run `make reproduce` and get a MATCH, so that I can publish independently that the national tally is arithmetically correct.
2. As a party technical advisor, I want to verify that a specific Form 34A's hash appears in the Merkle tree, so that I can advise my party whether to file a petition on form-tampering grounds.
3. As an IAB member, I want to commission a second implementation and compare outputs, so that I can publish an independent assurance that no single-vendor manipulation occurred.
4. As a Supreme Court IT expert, I want to reproduce the tally and sign an affidavit, so that the court has independent expert evidence of result integrity.
5. As a voter, I want to enter my participation token in M13 and confirm I was counted, so that I have personal evidence my presence was recorded.
6. As a data journalist, I want to download the input manifest and inspect every Form 34A hash, so that I can write a story confirming every form is accounted for.
7. As a security researcher, I want to find a discrepancy and claim the bug-bounty, so that the system is stress-tested by adversarial independent verification.
8. As an ELOG statistician, I want to re-aggregate Form 34Bs from the published inputs, so that I can cross-check IEBC's Form 34C without asking for data access.

---

## Data Model Additions

```python
class ReproducibilityBundle(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    election_phase: str  # "general_2027" or "by_election_2028_..."
    bundle_version: str  # semantic version
    manifest_hash_sha256: str
    form_count: int
    bundle_size_bytes: int
    published_at: datetime
    gazette_reference: str | None
    radio_bulletin_delivered: bool
    publication_channels: list[str]  # URLs of 5 channels
    expected_stage_hashes: dict[str, str]  # stage_name -> sha256

class ParticipationToken(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    token_sha256: str  # SHA-256 of the 256-bit random token
    station_code: str
    issued_at: datetime  # EAT
    kit_serial: str
    included_in_bundle: bool
    bundle_id: str | None

class BugBountySubmission(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    submitted_at: datetime
    submitter_handle: str  # HackerOne handle, pseudonymous
    discrepancy_type: str  # constituency | national | stage-N
    station_code: str | None
    constituency_code: str | None
    claimed_output_hash: str
    published_expected_hash: str
    reproduction_log_hash: str
    status: str  # pending | confirmed | rejected | awarded
    award_amount_kes: int | None
```

---

## API Surface

| Method | Path | Auth | Rate limit | Notes |
|---|---|---|---|---|
| GET | `/api/v1/bundle/latest` | None (public) | 100/s | Metadata for current bundle (not the bundle itself) |
| GET | `/api/v1/bundle/{phase}/manifest.json` | None (public) | 50/s | Signed input manifest |
| GET | `/api/v1/bundle/{phase}/bundle.tar.gz` | None (public) | 5/s per IP | Full bundle download; served from CDN |
| GET | `/api/v1/bundle/{phase}/expected/{stage}.sha256` | None (public) | 100/s | Per-stage expected output hash |
| POST | `/api/v1/participation/token/verify` | None (public) | 20/s per IP | Verify token presence in roll |
| GET | `/api/v1/merkle/root/history` | None (public) | 100/s | All published daily Merkle roots with timestamps |
| POST | `/api/v1/bugbounty/submit` | API key (HackerOne) | 1/s | Bug-bounty submission intake |
| GET | `/api/v1/bugbounty/{id}/status` | Submitter token | 10/s | Status of bounty submission |
| GET | `/api/v1/second-impl/agreement` | IAB JWT | 10/s | Second-implementation comparison results |

---

## Integration Points

**Upstream:**
- M10 (RTS Back-End) — signed Form 34A JSON + PDF feed
- M08 (RTS Device Client) — participation token generation
- P06 (HSM) — Merkle root signing, bundle manifest signing
- P05 (Audit log) — every publication event logged
- P04 (IAM) — IAB authenticated endpoints

**Downstream:**
- M13 (Public Verifier App) — participation token verification, bundle download instructions
- M15 (IAB Dashboard) — reproducibility audit feed
- M16 (Election Command Centre) — radio bulletin confirmation
- Kenya Government Printer API — daily Gazette Merkle root
- KBC Radio — 20:00 bulletin delivery
- IPFS node — decentralised Merkle root publication
- GitHub / IEBC container registry — image mirrors
- HackerOne — bug-bounty intake

---

## UI/UX — Key Screens

1. **Bundle landing page** (`iebc.go.ke/reproduce`) — current bundle status, download links, quick-verify instructions, `curl` one-liner
2. **Manifest explorer** — searchable table of all Form 34A entries; click any row to see hash, Merkle path, PDF link
3. **Stage-hash dashboard** — per-stage expected hash with green/amber status as stages complete
4. **Participation token lookup** — embedded in M13; also available as standalone page
5. **Bug-bounty status page** — public log of all submissions (anonymised) and resolution status
6. **Second-implementation report** — IAB-published comparison report embedded

---

## Compliance Mapping

| Requirement | Source |
|---|---|
| Verifiable election method (bit-reproducible) | Constitution Art. 86(a) |
| Transparent method | Constitution Art. 86(a); Raila v IEBC [2017] para 272 |
| Records retention >= 7 years | Elections Act s.44; Limitation of Actions Act s.4 |
| Open-source code publication | Strategy §XI.6 Undertaking Letter |
| No voter PII in public artefacts | DPA 2019 s.26, s.47 |
| Government Gazette publication | Kenya Gazette Act |
| IPR vests in IEBC | GCC 15.4 |

---

## Success Metrics and SLA

| Metric | Target | Sign-off |
|---|---|---|
| Full-pipeline bit-identical re-run rate | 100% on same inputs | IAB |
| Bundle available within X of Form 34C signing | <= 4 hours | IEBC ICT |
| Unique independent reproduction runs confirmed | >= 100 unique parties within 24 hours | Datacraft |
| Gazette publication SLA | 100% of daily windows during E-30 to E+30 | IEBC Chair |
| Radio bulletin delivery SLA | 100% of daily windows | IEBC Comms |
| Bug-bounty discrepancy count | 0 | IAB |
| Participation token issuance coverage | 100% of authenticated voters | IAB |
| Second-implementation agreement at result level | >= 99.99% | IAB |

---

## Test Strategy

- **Unit:** Each pipeline stage tested in isolation with known inputs and expected outputs. 100% coverage on aggregation arithmetic.
- **Determinism test:** Pipeline run twice in parallel on identical inputs; outputs must be bit-identical. Run in CI on every commit.
- **Multi-environment:** CI matrix: Ubuntu 24.04 x86-64, Ubuntu 24.04 ARM64 (via QEMU), macOS 15 x86-64. All three must produce identical hashes.
- **Bundle size test:** CI check that compressed bundle remains < 20 GB after each merge.
- **Gazette integration test:** Dry-run Gazette submission in staging environment at T-90.
- **Load test:** 1,000 concurrent bundle downloads; CDN failover to secondary and tertiary mirrors.
- **Participation token privacy audit:** Statistical analysis confirming tokens are uniformly distributed and unlinkable to voter identities. DPA 2019 DPIA sign-off.
- **Adversarial test (bug-bounty dry run):** Deliberately introduce a single-vote discrepancy in a test bundle and confirm the pipeline detects it before T-90.

---

## Rollout

| Phase | Date | Milestone |
|---|---|---|
| Pipeline architecture finalized | 2026-11-30 | Internal sign-off |
| Participation token design reviewed (DPA DPIA) | 2027-01-15 | ODPC acknowledgement |
| Second-implementation RFP issued to universities | 2027-02-01 | IAB commissions |
| Dry-run bundle generated from mock forms | 2027-06-10 (T-60) | Dry-run election |
| Bundle published (real forms) | 2027-08-10 to +4 hours | Form 34C signing |
| 15-year escrow deposit confirmed | 2027-08-15 | NCC Group certificate |

---

## Dependencies

- **M08** (RTS Device Client) — participation token generation on-device
- **M10** (RTS Back-End) — Form 34A feed into pipeline
- **M15** (IAB Dashboard) — reproducibility audit consumer
- **P05** (Audit log) — event logging
- **P06** (HSM) — Merkle root and bundle signing
- External: Government Printer API; KBC Radio SFTP; GitHub; IPFS; HackerOne; NCC Group escrow

---

## Risks and Mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | Non-determinism introduced by dependency update | All dependencies pinned with SHA-256; no automatic updates; CCB approval required for any dependency change |
| R2 | Bundle too large to download in Kenya in reasonable time | Hard CI size gate; CDN acceleration; IPFS peer-to-peer fallback |
| R3 | Gazette API failure delays daily publication | Manual fallback: PDF submitted to Government Printer via secure email; timestamp hash published on IAB domain immediately |
| R4 | Second-implementation identifies discrepancy | Immediate halt; manual adjudication by IAB; reproducibility bundle updated with explanation; full public disclosure |
| R5 | Participation token design challenged as tracking mechanism | Independent privacy audit pre-launch; DPIA published; token unlinkability formally proven and published |
