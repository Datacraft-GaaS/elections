# M21 — RLA Audit Tool (Risk-Limiting Audit)

**Module ID:** M21
**Name:** Risk-Limiting Audit Tool
**Runs on:** Central audit servers (Kenyan cloud) + IAB-issued audit laptops (offline-capable)
**Owner:** Datacraft
**Status:** Draft v2
**IPR:** Custom Software vests in the Commission per GCC 15.4; SHANGRLA framework components OSS (MIT/Apache 2.0)

## 1. Purpose

Provide a mathematically-guaranteed post-election audit mechanism using the SHANGRLA framework (Stark 2020). After Form 34C is published, sample physical ballots via cryptographic sortition seeded from a public multi-source random beacon, retrieve and tally the sample, and determine with probability >= (1 - alpha) whether the reported winner actually won. If the audit fails to confirm, a full recount under Elections Act s.80 is triggered automatically.

Strategy reference: §XIX.2 / §XV.1 / §XI.

## 2. User Personas

- **IAB Audit Commissioner** — initiates the audit, approves sample parameters, signs off on outcome
- **KEOG/ELOG Sampling Team Member** — physically retrieves sampled ballots under camera supervision
- **Party Agent (Audit Observer)** — witnesses ballot retrieval and tallying; can query sample manifest
- **Presiding Officer (at sampled stations)** — assists in locating and opening sampled ballot boxes
- **Datacraft Audit Engineer** — operates the statistical software; has no authority to change results
- **Independent Statistician (IAB-contracted)** — verifies sample calculations and statistical outputs
- **Supreme Court IT Registrar** — receives the signed audit report for petition-window filing

## 3. Functional Requirements

- **FR-M21-001** Ingest the published Form 34C JSON tally and its Merkle root as the starting point for the audit.
- **FR-M21-002** Implement ballot-polling RLA per the SHANGRLA framework (Stark 2020, "SHANGRLA: Sharper Discrepancy Bounds for Risk-Limiting Audits"). Wrap or integrate the upstream open-source implementation at github.com/pbstark/audit-tools. Publish all modifications open-source under OSS licence.
- **FR-M21-003** Compute sample size from user-supplied margin of victory (derived from Form 34C), risk limit alpha (default 0.05), and contest structure. Display the formula used, all inputs, and the computed sample size before drawing the sample.
- **FR-M21-004** Derive the random seed from five independent public entropy sources (NIST Randomness Beacon, UK National Lottery, BBC World Service shortwave hash, Kenya Gazette publication hash, SHA-256 of last Bitcoin block header at draw time). Record all five source values with fetch timestamps in the audit log before seed computation.
- **FR-M21-005** Generate a deterministic ballot manifest — specifying station code, ballot-box number, and sequential ballot position for each sampled ballot — from the seed alone. Any party with the five entropy values and the software can reproduce the manifest exactly.
- **FR-M21-006** Display the sample manifest as: (a) on-screen sortable table, (b) PDF download with QR code linking to published manifest hash, (c) machine-readable JSON signed by IAB HSM. Publish all three to the public audit portal within 30 minutes of seed draw.
- **FR-M21-007** Provide a ballot-retrieval checklist per sampling-team member: station, box, ballot position, expected seal numbers, GPS coordinates for station. QR-code scannable confirmation for each retrieved ballot.
- **FR-M21-008** Tally interface for retrieved ballots: sampling team member records which candidate each retrieved ballot votes for (or "undervote", "overvote", "unreadable"). Read-only display of Form 34A tallies for that station alongside.
- **FR-M21-009** Compute running SHANGRLA T-statistic after each batch of tallied ballots. Display current confidence level, remaining sample needed to confirm (or "sample exhausted — trigger recount"), and stopping condition status.
- **FR-M21-010** If at any point the T-statistic fails to confirm at the declared risk limit after the maximum planned sample is exhausted, generate a RECOUNT_REQUIRED event published immediately to the Commission Chair, IAB, and the public audit portal.
- **FR-M21-011** Generate a signed audit report upon completion: seed values, sample manifest hash, tally data, T-statistic trajectory, outcome (CONFIRMED / RECOUNT_REQUIRED), and all party-agent + IAB signatures. Publish report on the public audit portal and submit to the Supreme Court IT Registrar.
- **FR-M21-012** Offline mode: audit laptop can complete the statistical computation and tally entry without network. Syncs signed results on reconnect.
- **FR-M21-013** All computation is reproducible from published inputs. Publish a reproduce-me script alongside every audit.
- **FR-M21-014** Support multi-contest audits (presidential + gubernatorial + senatorial + MP + women-rep + MCA) with independent alpha per contest.
- **FR-M21-015** Cryptographic sortition log appended to the main Merkle audit chain (M10/P05) within 1 hour of seed draw.

## 4. Non-Functional Requirements

- **NFR-M21-001** Sample manifest computation <= 60 seconds for 55,393 stations on reference hardware (2 vCPU, 8 GB RAM).
- **NFR-M21-002** Software reproducibility: given identical inputs, bit-identical output on any POSIX-compatible system with Python 3.11+.
- **NFR-M21-003** Availability of public audit portal during 30-day petition window: >= 99.99% (4 hours annual downtime budget).
- **NFR-M21-004** Audit log immutability: append-only; every entry SHA-256-chained; any deletion or modification detectable.
- **NFR-M21-005** WCAG 2.2 AA for all auditor-facing and public-facing screens.
- **NFR-M21-006** Languages: EN + SW (auditor-facing); EN + SW + per-county Kenyan language (public audit portal).
- **NFR-M21-007** DPA 2019 compliance: audit log contains no voter-identifiable data; sampled-ballot records contain station + ballot-position only.
- **NFR-M21-008** Signed outputs: every published artefact carries an HSM-generated detached ECDSA-P384 signature. Verification instructions published in plain language.
- **NFR-M21-009** Security: audit software runs in an air-gapped environment during seed draw and sample computation; external network access via one-way data diode for public portal publication.
- **NFR-M21-010** Scalability: supports audit of up to 1,000,000 sampled ballots without code modification.

## 5. User Stories

1. As an IAB Audit Commissioner, I can set the risk limit to 5% and enter the margin of victory, so the system computes the minimum sample size and I can approve it before drawing.
2. As a KEOG sampling-team member, I receive a GPS-linked checklist with seal numbers so I can locate and retrieve exactly the right ballot box at each station.
3. As a party agent, I can verify the seed values from the five public entropy sources myself using publicly available tools, so I know the sample was not cherry-picked.
4. As an independent statistician, I can run the reproduce-me script on my own laptop and get an identical audit report, so I can certify the result independently.
5. As an IAB Audit Commissioner, I see the T-statistic update after each batch of 100 tallied ballots so I know whether we have reached statistical confidence.
6. As a Supreme Court IT Registrar, I receive the signed audit report at T+7 in machine-readable and PDF form so it is available for any petition filed within the statutory window.
7. As the public, I can download the full seed values, sample manifest, tally data, and statistical software from the public audit portal so any citizen can reproduce the audit.
8. As a returning officer at a sampled station, I receive advance notice (48 hours) that my station is in the sample so I can ensure ballot boxes remain sealed and accessible.
9. As a Datacraft Audit Engineer, I cannot modify any tally entry after a party-agent witness has counter-signed the retrieval record.
10. As an IAB member, I can trigger a RECOUNT_REQUIRED event with one click if the statistical stopping condition is not met, and the event is immediately visible on the public portal.

## 6. Data Model Additions

```python
class AuditRun(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                        # uuid7str()
    election_id: str
    contest_id: str
    risk_limit_alpha: float        # default 0.05
    margin_of_victory: float       # derived from Form 34C
    seed_nist: str                 # NIST Randomness Beacon value
    seed_uk_lottery: str
    seed_bbc_hash: str
    seed_kenya_gazette_hash: str
    seed_bitcoin_block_hash: str
    seed_composite: str            # XOR of five sources, hex
    seed_drawn_at: datetime
    sample_size_target: int
    manifest_hash: str             # SHA-256 of published manifest JSON
    status: Literal["PENDING","DRAWING","ACTIVE","CONFIRMED","RECOUNT_REQUIRED"]
    outcome_signed_at: datetime | None
    report_url: str | None
    merkle_inclusion_proof: str | None
    created_at: datetime

class SampledBallot(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    audit_run_id: str
    station_code: str
    ballot_box_number: int
    ballot_position_in_box: int    # 1-indexed
    retrieval_confirmed_at: datetime | None
    retrieval_confirmed_by: str | None  # sampling-team member uuid
    tally_contest_id: str
    tally_candidate_id: str | None      # null if undervote/overvote/unreadable
    tally_outcome: Literal["VOTE","UNDERVOTE","OVERVOTE","UNREADABLE"]
    party_agent_witness_id: str | None
    party_agent_signed_at: datetime | None
    t_statistic_after: float | None

class AuditEntropySource(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    audit_run_id: str
    source_name: str
    fetch_url: str
    raw_value: str
    fetched_at: datetime
    fetched_by: str   # engineer uuid
```

## 7. API Surface

| Method | Path | Auth | Rate limit | Description |
|--------|------|------|------------|-------------|
| POST | /v1/audit/runs | IAB_ADMIN JWT | 10/hr | Create new audit run; supply election_id, contest_id, alpha, margin |
| GET | /v1/audit/runs/{id} | Public (read) | 1000/min | Fetch audit run status and metadata |
| POST | /v1/audit/runs/{id}/draw-seed | IAB_ADMIN JWT + HSM | 1/run | Trigger entropy fetch and seed computation |
| GET | /v1/audit/runs/{id}/manifest | Public | 500/min | Download sample manifest (JSON + PDF links) |
| GET | /v1/audit/runs/{id}/manifest/verify | Public | 500/min | Returns manifest Merkle inclusion proof |
| POST | /v1/audit/runs/{id}/ballots/{ballot_id}/tally | AUDITOR JWT | 200/min | Record tally for one sampled ballot |
| POST | /v1/audit/runs/{id}/ballots/{ballot_id}/witness | PARTY_AGENT JWT | 200/min | Party-agent counter-signature on retrieval |
| GET | /v1/audit/runs/{id}/tstatistic | IAB_ADMIN JWT | 300/min | Current T-statistic + confidence level |
| POST | /v1/audit/runs/{id}/close | IAB_ADMIN JWT + HSM | 1/run | Seal audit run; generate signed report |
| GET | /v1/audit/runs/{id}/report | Public | 500/min | Download signed audit report (PDF + JSON) |
| GET | /v1/audit/entropy-sources | Public | 200/min | List entropy sources with fetch timestamps |

All endpoints: TLS 1.3, ECDSA-P384 response signing, OpenAPI 3.1 spec-first.

## 8. Integration Points

- **M10 RTS Back-End** — imports Form 34C tally as audit input; appends audit Merkle nodes to the main chain
- **P05 Merkle/Audit Chain** — all audit events anchored here
- **P06 HSM** — seeds audit report signature; IAB Commissioner HSM card required for seed draw and report close
- **NIST Randomness Beacon** — public entropy source (fetched at draw time; value published)
- **M11 Public Portal** — audit portal is a dedicated sub-path of the public portal
- **M23 Grievance Channels** — any grievance referencing a specific sampled ballot can be cross-referenced against audit record

## 9. UI/UX Key Screens

1. **Audit Setup** — contest selector, alpha slider (default 0.05, range 0.01-0.10), margin field, computed sample size, approval button for IAB Commissioner.
2. **Seed Draw Dashboard** — live status of each entropy source fetch (pending/fetched/failed), raw values when complete, composite seed display, draw confirmation button (requires HSM card tap).
3. **Sample Manifest Viewer** — sortable/filterable table of all sampled ballots; PDF and JSON download buttons; Merkle inclusion proof link.
4. **Ballot Retrieval Checklist** — mobile-responsive; one card per sampled ballot with station name, GPS link, box number, seal number; scan-to-confirm QR flow.
5. **Tally Entry** — one ballot at a time; candidate selector per contest (radio buttons with candidate name + photo); "undervote / overvote / unreadable" options; party-agent counter-sign button.
6. **T-statistic Monitor** — real-time chart of T-statistic vs stopping boundary; current confidence %; projected ballots remaining; CONFIRMED / RECOUNT_REQUIRED banner.
7. **Audit Report** — summary card + full downloadable PDF + JSON; Merkle proof; party-agent and IAB signature badges.

## 10. Compliance Mapping

| Requirement | Source |
|-------------|--------|
| Post-election recount authority | Elections Act 2011 s.80 — full recount triggered on audit failure |
| Transparent and verifiable election | Constitution Art. 86(a) — reproducible audit supports verifiability |
| Public access to election information | Access to Information Act 2016 s.4 — all audit artefacts published |
| Data minimisation | DPA 2019 s.26 — ballot records contain no voter-identifiable data |
| Immutable audit log | Elections Act 2011 s.39; Election Technology Regulations 2017 |
| IAB commission mandate | Tender the tender specification |

References: Stark 2008 ("Conservative Statistical Post-Election Audits", JASA 103(483)); Stark 2020 ("SHANGRLA"); Lindeman & Stark 2012 ("A Gentle Introduction to Risk-Limiting Audits", IEEE Security and Privacy).

## 11. Success Metrics + SLA

| Metric | Target | Measurement | Sign-off |
|--------|--------|-------------|----------|
| Audit confirmed at alpha = 0.05 for all six contests | 100% | T-statistic output | IAB |
| Time from Form 34C publication to seed draw | <= 48 hours | Timestamp delta | IAB Chair |
| Sample manifest published after seed draw | <= 30 minutes | Timestamp delta | Datacraft Technical |
| Ballot retrieval completion | >= 98% of sample within 7 days | Retrieval confirmation log | IAB |
| Audit report delivered to Supreme Court IT Registrar | <= T+7 days post-election | Signed receipt | IAB Chair |
| Public audit portal uptime during 30-day petition window | >= 99.99% | Uptime monitor | Datacraft SRE |
| Reproduce-me script producing identical results | 100% on reference platform | CI test suite | Datacraft Technical |
| Party-agent witness coverage | 100% of sampled ballots | Witness log | IAB |

## 12. Test Strategy

- **Unit:** statistical functions (T-statistic, sample size formula, seed XOR); Merkle append; entropy source parsers.
- **Integration:** end-to-end audit run with synthetic Form 34C; seed draw against NIST beacon sandbox; manifest generation reproducibility.
- **E2E:** full simulated audit with 500 synthetic ballots; outcome matches manual calculation.
- **Security:** penetration test on audit portal API; attempt to modify tally entry post-witness-signature; attempt to alter seed values.
- **Reproducibility:** automated CI runs reproduce-me script on every release and compares byte-for-byte.
- **Accessibility:** WCAG 2.2 AA audit on all screens (axe-core + manual screen reader on NVDA + ChromeVox).
- **Load:** 50,000 concurrent public manifest downloads during simulated audit.
- **Chaos:** network partition during tally entry (offline mode); recovery and sync validation.

## 13. Rollout

| Milestone | Date | Description |
|-----------|------|-------------|
| Software complete + open-source published | 2026-11-30 | Code on public repo; licence confirmed |
| Pilot RLA on 2026 by-election | 2027-01-15 | Real ballot-polling RLA on constituency by-election |
| IAB audit team training | 2027-03-01 | Two cohorts; KEOG/ELOG sampling-team training |
| Dry-run audit on simulated election data | 2027-05-01 | Full-scale simulation with party agents |
| Pre-election audit software deployed | 2027-07-01 | Production deploy T-40 from election day 2027-08-10 |
| Seed draw | 2027-08-12 | T+2 post Form 34C publication |
| Audit complete + report published | 2027-08-19 | T+9; before petition window closes |

## 14. Dependencies

- P05 Merkle/Audit Chain — for audit log anchoring
- P06 HSM — for seed draw and report signing
- M10 RTS Back-End — for Form 34C import
- M11 Public Portal — for public audit portal hosting
- NIST Randomness Beacon (external) — for entropy source 1
- IAB appointment — external governance dependency (T-180)
- KEOG/ELOG MoU for sampling teams — external (T-120)
- Elections Act s.80 regulatory clarity that RLA-triggered recount is legally operative — legal dependency (T-180)

## 15. Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Physical ballot boxes not accessible at sampled stations (boxes in transit, broken seals) | Medium | High | 48-hour advance notice per station; dual-chain of custody tracking; replacement-ballot protocol from next-closest sampled ballot |
| NIST Randomness Beacon unavailable at seed draw time | Low | Medium | 15-minute retry window; fallback to cached NIST value from prior 24 hours with published rationale; other four sources still provide entropy |
| Party agents refuse to witness or sign sampled-ballot tallies | Low | Medium | MoU with political parties signed pre-election; IAB observer replaces party-agent witness with auditor-witnessed affidavit |
| Statistical software bug produces incorrect T-statistic | Low | High | Independent implementation by IAB-contracted statistician running in parallel; code published open-source T-90 for community review; reproduce-me CI on every commit |
| Petition filed claiming audit itself was manipulated | Low | Medium | Multi-source entropy + open-source software + party-agent signatures + independent statistician + Supreme Court IT Registrar receipt constitute a complete evidentiary record |
