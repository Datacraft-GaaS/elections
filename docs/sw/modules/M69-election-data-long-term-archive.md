# M69 — Election Data Long-Term Archive

**Module ID:** M69
**Name:** Election Data Long-Term Archive
**Owner:** Datacraft Limited — Platform Engineering Team
**Status:** Draft v1
**IP Classification:** Custom Software — IPR vests in IEBC per GCC 15.4

**Strategy traceback:** Elections Act §44 (7-year minimum); M14 Reproducibility Bundle (pattern); Kenya National Archives Act Cap. 19

---

## Purpose

15-year retention archive interface sitting above operational storage systems. Manages the transition of election data from operational databases into long-term preservation formats, integrates with Kenya National Archives and Kenya National Archives and Documentation Service (KNADS), provides format-migration pipelines as storage formats age, issues signed preservation attestations, and exposes a public-access API for researchers, historians, and academic institutions. Distinct from M14 Reproducibility Bundle (which handles near-term reproducibility) and from operational M10/M20 storage. M69 is the terminal end of the data lifecycle.

---

## User Personas

| Persona | Access | Primary task |
|---|---|---|
| IEBC Archivist | Write | Trigger ingestion, manage format migrations, issue attestations |
| KNADS Archivist | Write (KNADS-designated collection) | Receive national-archive deposit, confirm accession |
| Academic Researcher | Read (public API, after embargo) | Query archived datasets, download research extracts |
| Data Journalist | Read (public API) | Access historical election data for investigative work |
| ODPP / Judicial Officer | Read (restricted, on order) | Access archived evidentiary material within retention window |
| IAB Member | Read (all tiers) | Audit preservation attestations and format-migration logs |
| Auditor-General | Read (all tiers, batch extract) | Confirm 7-year retention compliance |
| M14 (Reproducibility Bundle) | API consumer | Long-term bundle deposit at T+4h; retrieval on demand |
| M66 (Financial Forensics) | API consumer | Panel dataset deposit for 15-year cross-cycle analysis |

---

## Functional Requirements

### FR-69.01 — Data source registry
Register every operational data source eligible for archival: source system ID, data category (results / voter-roll snapshot / chain-of-custody / financial-disclosure / audit-log / reproducibility-bundle / financial-forensics-panel), retention period (7 / 10 / 15 years), classification (public / restricted / restricted-judicial-only), PII flag, lawful basis for retention.

### FR-69.02 — Ingestion pipeline
Scheduled and triggered ingestion jobs pull data from operational systems at defined milestones:
- T+4h: Reproducibility bundle deposit (from M14)
- T+24h: Full signed results dataset (from M10)
- T+30d: Chain-of-custody final reconciliation (from M20 + M63)
- T+90d: Campaign finance final disclosure (from M22)
- T+90d: Financial forensics panel snapshot (from M66)
- Annual: Voter-roll snapshot (from M04)
- Annual: Audit log extract (from P05)

Each ingestion run: record count, hash manifest, source system version, ingest timestamp.

### FR-69.03 — Format migration
Format-migration at 5-year intervals. Pipeline: render current format to open canonical (CSV + JSON-LD for structured data; PDF/A-3 for documents; FLAC for audio; BagIt container); verify byte-count + hash against pre-migration manifest; generate signed migration attestation (IEBC Archivist + KNADS counter-signature). Migration log retained alongside data.

### FR-69.04 — KNADS deposit
Designated collections deposited to KNADS via SFTP + PGP. KNADS accession number recorded. Deposit receipt (signed PDF) stored in M69 attestation store.

### FR-69.05 — Signed preservation attestation
At each checkpoint (T+1y, T+5y, T+10y, T+15y): IEBC Archivist generates attestation signed by HSM key + KNADS counter-signature. Attestation: manifest hash, current format, storage locations, KNADS accession, next migration date. Published to M11 (public collections) and IAB Dashboard (all).

### FR-69.06 — Public-access API
After embargo (public: T+1y; restricted: T+5y after judicial close; PII: never public): read-only API. Endpoints: collection browser, dataset query (year / constituency / category), file download, citation generator (BibTeX / DataCite DOI). Rate-limited: 10 req/min per authenticated researcher.

### FR-69.07 — DOI registration
DataCite DOI for each public collection version. DOI resolves to M69 landing page. Citation format in API response.

### FR-69.08 — Expiry and destruction
At retention end: Archivist reviews. PII collections: DPA 2019 §41 cryptographic erasure + destruction certificate + ODPC notification. Non-PII: may be extended by IEBC resolution or transferred to KNADS permanently. Signed destruction certificate for every event.

### FR-69.09 — Research access application
Restricted-tier: researcher submits institution, research purpose, data categories, IRB clearance reference. IEBC Research Committee decides within 30 days. Approved: time-limited DUA + watermarked downloads. Denied: written reason + appeal to Commissioner.

### FR-69.10 — Cross-cycle dataset linking
Cross-cycle link table: constituency code + station code across 2017 / 2022 / 2027 cycles. Enables longitudinal queries without exposing internal IDs. DOI-linked per cycle.

---

## Non-Functional Requirements

### NFR-69.01 — Retention
15-year maximum retention period; 7-year minimum per Elections Act §44. Storage: Nairobi Tier III DC + KNADS physical site + offsite cold storage (tape). Three-copy strategy (3-2-1: 3 copies, 2 media types, 1 offsite).

### NFR-69.02 — Format longevity
Open formats only for long-term storage: CSV, JSON-LD, PDF/A-3, FLAC, BagIt, PREMIS metadata. Proprietary database dumps kept as secondary copy only. Format review every 5 years per ISO 14721 (OAIS model).

### NFR-69.03 — Integrity
Every archived file: SHA-256 stored at ingest and re-verified annually. Any checksum failure: P1 alert to IEBC Archivist and IAB. WORM storage for primary archive (object-lock with 15-year retention policy).

### NFR-69.04 — DPA 2019
PII-containing collections (voter rolls, biometric logs): DPA 2019 §26 purpose limitation; access restricted to ODPP/judicial-only tier; cryptographic erasure at retention end. Non-PII collections: public after embargo. DPIA completed for each PII collection.

### NFR-69.05 — Availability
Public-access API: 99.5%. Archive ingest jobs: batch, best-effort within defined SLA windows. WORM primary: 99.99% read availability.

### NFR-69.06 — WCAG 2.2 AA
Researcher portal and public landing pages meet WCAG 2.2 AA. EN and SW; academic API documentation in EN.

---

- **NFR-M69-MONITORING** Observability: OpenTelemetry traces exported to the central collector; module Grafana dashboard named `iebc-iems-M69`; alert ruleset ID `iems-M69-alerts` in the Alertmanager configuration. Dashboard and ruleset must exist at OAT gate per FAT-07. See sw/runbooks/ for dashboard template.

## User Stories

1. As an IEBC Archivist, I want to trigger the M14 bundle ingest at T+4h and receive a signed attestation, so that reproducibility data is in long-term storage before petition windows open.
2. As a KNADS Archivist, I want to receive the designated collection via SFTP+PGP and issue an accession number, so that Kenya National Archives holds an independent copy.
3. As an Academic Researcher, I want to query archived results datasets via DOI-referenced API, so that I can conduct longitudinal constituency-level studies.
4. As an IAB Member, I want to verify the preservation attestation hash at the T+1y checkpoint, so that I can confirm no data has been modified since ingest.
5. As the Auditor-General, I want a signed batch extract confirming 7-year retention compliance for financial disclosure records, so that the PFMA audit is satisfied.
6. As a Data Journalist, I want to download constituency-level results for every election since 2017 as a single CSV, so that I can run a cross-cycle analysis.
7. As a Researcher applying for restricted-tier access, I want to submit my IRB clearance and research purpose, so that IEBC Research Committee can review within 30 days.
8. As an IEBC Archivist, I want an alert when any archived file fails its annual SHA-256 checksum, so that I can restore a corrupted archive before the next check cycle.
9. As a Judicial Officer, I want to retrieve a signed copy of chain-of-custody records for a specific station, so that I have certified evidentiary material for petition proceedings.
10. As an ODPC Officer, I want a destruction certificate when a PII collection reaches retention end, so that DPA 2019 §41 compliance is documented.
11. As a PhD Student, I want to cite the archived dataset using its DataCite DOI, so that my research is reproducible and the citation is stable.

---

## Data Model

```python
from pydantic import BaseModel, ConfigDict
from datetime import datetime, date

class DataSource(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    system_id: str  # source module ID (e.g. "M14", "M10")
    category: str  # results | voter_roll | chain_of_custody | financial | audit_log | reproducibility | forensics
    retention_years: int  # 7 | 10 | 15
    classification: str  # public | restricted | restricted_judicial
    pii_flag: bool
    lawful_basis: str  # DPA 2019 reference

class ArchiveCollection(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    source_id: str
    election_cycle: str  # "2027_general"
    ingested_at: datetime
    record_count: int
    manifest_sha256: str
    format: str  # csv | json_ld | pdf_a3 | flac | bagit
    storage_locations: list[str]  # URIs
    knads_accession: str | None
    doi: str | None
    embargo_lifts: date | None
    retention_expires: date
    status: str  # active | migrating | expired | destroyed

class PreservationAttestation(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    collection_id: str
    checkpoint: str  # T+1y | T+5y | T+10y | T+15y
    manifest_sha256: str
    format_at_checkpoint: str
    storage_locations: list[str]
    knads_accession: str
    next_migration_date: date
    iebc_archivist_signature: str
    knads_counter_signature: str | None
    attested_at: datetime

class FormatMigration(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    collection_id: str
    from_format: str
    to_format: str
    migrated_at: datetime
    pre_migration_sha256: str
    post_migration_sha256: str
    record_count_match: bool
    migration_log_url: str
    archivist_id: str

class ResearchAccessRequest(BaseModel):
    model_config = ConfigDict(extra='forbid')
    id: str  # UUID7
    researcher_name: str
    institution: str
    research_purpose: str
    data_categories: list[str]
    irb_reference: str | None
    submitted_at: datetime
    status: str  # pending | approved | denied | expired
    approved_until: date | None
    dua_url: str | None
    denial_reason: str | None
```

---

## API (OpenAPI 3.1)

```yaml
openapi: "3.1.0"
info:
  title: M69 Election Data Long-Term Archive API
  version: "1.0.0"
paths:
  /api/v1/archive/collections:
    get:
      summary: Browse archive collections
      parameters:
        - name: cycle
          in: query
          schema: {type: string}
        - name: category
          in: query
          schema: {type: string}
        - name: classification
          in: query
          schema: {type: string}
      responses:
        "200": {description: List of ArchiveCollection (public fields only for unauthenticated)}
  /api/v1/archive/collections/{id}:
    get:
      summary: Get collection metadata and manifest
      responses:
        "200": {description: ArchiveCollection with attestation links}
  /api/v1/archive/collections/{id}/download:
    get:
      summary: Download collection (authenticated; embargo enforced)
      security: [{BearerAuth: []}]
      responses:
        "200":
          description: Collection archive file
          content:
            application/octet-stream:
              schema: {type: string, format: binary}
        "403": {description: Embargo active or access not granted}
  /api/v1/archive/ingest:
    post:
      summary: Trigger ingest from source system (IEBC Archivist)
      security: [{BearerAuth: []}]
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                source_id: {type: string}
                cycle: {type: string}
                trigger: {type: string}
      responses:
        "202": {description: Ingest job queued}
  /api/v1/archive/attestations/{collection_id}:
    get:
      summary: Get preservation attestations for a collection
      responses:
        "200": {description: List of PreservationAttestation}
  /api/v1/archive/research-access:
    post:
      summary: Submit research access application
      security: [{BearerAuth: []}]
      requestBody:
        required: true
        content:
          application/json:
            schema: {$ref: "#/components/schemas/ResearchAccessRequest"}
      responses:
        "201": {description: Application submitted}
  /api/v1/archive/citation/{id}:
    get:
      summary: Get DataCite DOI citation for a collection
      parameters:
        - name: format
          in: query
          schema: {type: string, enum: [bibtex, datacite, ris]}
      responses:
        "200": {description: Citation in requested format}
  /api/v1/archive/checksum-report:
    get:
      summary: Annual checksum verification report (IAB, Auditor-General)
      security: [{BearerAuth: []}]
      responses:
        "200": {description: Per-collection checksum status}
```

---

## Integrations

**Upstream (data sources):**
- M14 (Reproducibility Bundle) — bundle deposit at T+4h
- M10 (RTS Back-End) — signed results dataset at T+24h
- M20 (IMS) + M63 (Ballot Serial Chain) — chain-of-custody at T+30d
- M22 (Campaign Finance) — final disclosures at T+90d
- M66 (Financial Forensics) — panel dataset at T+90d
- M04 (VPRS) — annual voter-roll snapshot
- P05 (Audit log) — annual extract
- P06 (HSM) — attestation signing

**Downstream (consumers):**
- KNADS — physical deposit via SFTP+PGP
- DataCite — DOI registration
- M11 (Public Portal) — public landing pages and collection browser
- M15 (IAB Dashboard) — attestation audit feed
- Auditor-General — batch compliance extract
- ODPC — destruction certificate notification
- Academic researcher portal (M69 public API)

---

## UI/UX — Key Screens

1. **Collection browser** (public) — filter by cycle, category; table view with classification badge, embargo status, DOI link; download button (embargo-aware); WCAG 2.2 AA; EN/SW
2. **Researcher portal** — application form; DUA download; watermarked download centre; usage reporting form
3. **Archivist dashboard** — ingest job queue; checksum status per collection; format-migration scheduler; expiry calendar; attestation generation workflow
4. **Attestation viewer** (public/IAB) — per-collection attestation history; manifest hash; KNADS accession; HSM signature verification link
5. **KNADS deposit tracker** (Archivist) — per-collection deposit status; accession number entry; receipt PDF upload

---

## Compliance Mapping

| Requirement | Source |
|---|---|
| Minimum 7-year retention | Elections Act §44 |
| National archives deposit | Kenya National Archives Act Cap. 19 |
| OAIS-compliant long-term preservation | ISO 14721 (M69 implementation follows) |
| PII destruction at retention end | DPA 2019 §41 |
| ODPC notification on personal data destruction | DPA 2019 §41(3) |
| Researcher access (public interest) | DPA 2019 §31 |
| DPIA for PII collections | DPA 2019 §31 |
| Records open to public after defined period | Access to Information Act 2016 §6 |

---

## Success Metrics and SLA

| Metric | Target | Sign-off |
|---|---|---|
| M14 bundle ingest completed by T+4h | 100% | IEBC Archivist |
| Annual SHA-256 checksum pass rate | 100% | IAB |
| KNADS deposit completed within 90 days of election | 100% | KNADS |
| DOI registration for public collections | 100% within T+1y | IEBC Research |
| Research access applications decided within 30 days | >= 95% | IEBC Research Committee |
| PII destruction certificates issued at retention end | 100% | IEBC DPO |
| Preservation attestation issued at each checkpoint | 100% | IEBC Archivist |
| Format migration completed before prior format unsupported | 100% | Datacraft SRE |

---

## Test Strategy

- **Unit:** Checksum verification, format migration pipeline (pre/post hash match), retention expiry calculation, embargo lift date logic
- **Integration:** M14 bundle ingest, M10 results ingest, KNADS SFTP deposit, DataCite DOI registration, P06 HSM attestation signing
- **Integrity:** Simulate bitrot (corrupt one byte in archive); confirm checksum failure alert fires within 24h of next annual check
- **Format migration:** Migrate a 2022-cycle CSV to JSON-LD; verify round-trip equivalence (field counts, value checksums)
- **Embargo enforcement:** Attempt public API download of a restricted collection before embargo lift; must return 403
- **WORM storage:** Attempt overwrite of an ingested collection; must fail at storage layer
- **DPA compliance:** Confirm no PII-containing collection accessible via public API; automated scan confirms redaction

---

## Rollout

| Phase | Date | Milestone |
|---|---|---|
| WORM storage provisioned (Nairobi DC) | 2026-12-01 | Primary + tape ready |
| KNADS MOU and deposit protocol agreed | 2027-01-15 | Deposit channel live |
| DataCite membership and DOI prefix registered | 2027-02-01 | Citation infrastructure |
| M14 and M10 ingest connectors built | 2027-04-01 | Ingest tested in staging |
| Researcher portal and public API live | 2027-05-01 | Pre-election data browsable |
| Election day ingest cycle begins (T+4h) | 2027-08-10 (T-0 + 4h) | M14 bundle ingested |
| T+30d chain-of-custody ingest | 2027-09-09 | M20/M63 snapshot |
| T+90d financial and forensics ingest | 2027-11-08 | M22/M66 snapshot |
| T+1y first preservation attestation | 2028-08-10 | IAB and KNADS sign-off |

---

## Dependencies

- **M14** (Reproducibility Bundle) — bundle deposit
- **M10** (RTS Back-End) — results dataset
- **M20** (IMS) + **M63** (Ballot Serial Chain) — chain-of-custody
- **M22** (Campaign Finance) — financial disclosures
- **M66** (Financial Forensics) — panel dataset
- **M04** (VPRS) — voter-roll snapshot
- **M15** (IAB Dashboard) — attestation audit
- **P05** (Audit log), **P06** (HSM), **P04** (IAM)
- External: KNADS, DataCite, ODPC, Auditor-General

---

## Risks and Mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | KNADS lacks capacity for digital deposit at required volume | Pilot deposit with KNADS at T-180; technical capacity-building included in IEBC-KNADS MOU; M69 holds primary copy independently |
| R2 | Open format becomes obsolete before 15-year horizon | 5-year format review cycle; OAIS model migration commitment; BagIt container preserves original alongside migrated format |
| R3 | Storage provider bitrot or media failure | 3-2-1 storage strategy; annual checksum; P1 alert on any failure; tape as third copy with 30-year lifespan |
| R4 | Researcher circumvents embargo via API enumeration | Authenticated access only post-embargo; embargo enforcement at data layer; API rate-limited; usage logged |
| R5 | DPA 2019 destruction obligation conflicts with KNADS retention | KNADS deposit agreement specifies PII-redacted version for long-term public collection; PII version held only by IEBC under DPA 2019 lawful basis; destruction covers IEBC copy only |

---

## Cross-References

M14 (bundle deposit), M10 (results), M20 (IMS chain-of-custody), M63 (ballot reconciliation), M22 (financial disclosures), M66 (forensics panel), M04 (voter-roll snapshot), M15 (IAB attestation audit).

---

## Strategy Traceability

| Section | Requirement |
|---|---|
| Elections Act §44 | 7-year minimum records retention |
| Kenya National Archives Act Cap. 19 | Mandatory national archive deposit |
| ISO 14721 (OAIS) | Open Archival Information System model |
| DPA 2019 §41 | Destruction of personal data at retention end |
| DPA 2019 §31 | Public-interest research access exception |
| Access to Information Act 2016 §6 | Public access after defined retention period |
| M14 FR-14.12 | 15-year reproducibility commitment integration |

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
