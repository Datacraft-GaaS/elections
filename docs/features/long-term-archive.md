# Long-term archive (M69)

Module M69 (`long-term-archive`) owned by `support-admin-svc`. Creates, preserves, and manages BagIt-format archive collections (BagIt v1.0, RFC 8493, Kunze and Littman, IETF 2018) for election records in compliance with the OAIS reference model (ISO 14721:2012). Attests preservation integrity using Preservation Attestations, issues Destruction Certificates for records past retention, mints DataCite DOIs (DataCite Metadata Schema v4.4, ISO 26324:2022) for publicly citable election datasets, and coordinates KNADS handover under the Public Archives and Documentation Service Act.

## Contents

1. What Long-Term Archive does (vs M14 Reproducibility Bundle)
2. Data plane (schema)
3. Standards and formats
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What Long-Term Archive does (vs M14 Reproducibility Bundle)

- **M14** (Reproducibility Bundle) = a self-contained, immediately downloadable package of the post-election results and audit data, assembled within days of the election for statutory publication and public scrutiny.
- **M69** = long-term preservation: takes the M14 bundle (and other election records) and manages their preservation over years and decades, including format migration, integrity monitoring, DOI minting, and eventual KNADS handover or destruction.

M14 is about immediate transparency. M69 is about institutional memory and archival permanence.

## 2. Data plane

Five aggregates (per `services/support-admin-svc/crates/archive/src/models.rs` and the M69 spec):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **ArchiveCollection** | Top-level archive unit: `collection_id`, `election_cycle_id`, `content_type` (RESULTS / AUDIT_TRAIL / VOTER_REGISTER / FINANCIAL / LEGAL / REPRODUCIBILITY_BUNDLE / TRAINING_RECORDS), `bag_url` (MinIO path to BagIt bag), `bag_checksum` (SHA-256 of bag), `doi` (DataCite DOI, e.g. `10.XXXXX/eel-2027-ke-results`), `retention_years`, `retention_expiry`, `status` (INGESTED / PRESERVED / DOI_MINTED / KNADS_TRANSFERRED / DESTROYED). | A |
| **IngestJob** | BagIt bag creation job: `job_id`, `collection_id`, `source_module` (M-number), `source_query_params` (JSONB), `bag_created_at`, `bag_manifest_alg` (SHA-256), `bag_oxum` (payload octet count + stream count, BagIt §2.2.3), `completed_at`, `status` (PENDING / RUNNING / COMPLETE / FAILED). | B |
| **PreservationAttestation** | Periodic integrity check: `attestation_id`, `collection_id`, `attested_at`, `bag_checksum_verified` (bool), `oxum_verified` (bool), `format_migration_needed` (bool), `attested_by_officer`, officer HSM signature. | A |
| **DestructionCertificate** | Authorised destruction record: `cert_id`, `collection_id`, `destruction_basis` (RETENTION_EXPIRED / LEGAL_ORDER / KNADS_DIRECTED), `approved_by_commissioner`, `destroyed_at`, commissioner HSM signature, `destruction_witness` (second officer). | A |
| **KNADSHandover** | Transfer to Kenya National Archives and Documentation Service: `handover_id`, `collection_id`, `knads_ref_number`, `transfer_method` (PHYSICAL_MEDIA / SFTP / API), `transferred_at`, `knads_acknowledgement_at`. | B |

## 3. Standards and formats

| Standard | Role in M69 |
|---|---|
| **BagIt v1.0 (RFC 8493, Kunze and Littman, IETF 2018)** | Packaging format for all archive collections. Bags include `bagit.txt`, `bag-info.txt`, `manifest-sha256.txt`, `tagmanifest-sha256.txt`, and the payload under `data/`. |
| **OAIS (ISO 14721:2012)** | Reference model: M69 implements Submission Information Packages (SIP, assembled by `IngestJob`), Archival Information Packages (AIP, stored `ArchiveCollection`), and Dissemination Information Packages (DIP, assembled on DOI download request). |
| **DataCite Metadata Schema v4.4 (ISO 26324:2022)** | DOI metadata format. `ArchiveCollection` rows with `content_type IN (RESULTS, REPRODUCIBILITY_BUNDLE)` get DOIs minted via DataCite API. Metadata maps to Schema v4.4 `resourceType = Dataset`. |
| **SHA-256 (NIST FIPS 180-4, 2015)** | Bag manifest and checksum algorithm. MD5 is not used. |
| **PRONOM file format registry (National Archives UK)** | Format migration decisions reference PRONOM identifiers for each payload file format; `format_migration_needed` flag is set when a PRONOM identifier is on the "at risk" list. |

## 4. Pipeline

Files on disk under `services/support-admin-svc/crates/archive/src`: `lib.rs`, `models.rs`, `ingest.rs`, `preservation.rs`, `doi.rs`, `destruction.rs`, `knads.rs`, `routes/`.

```
election cycle certified (eel.rts.results_certified.v1 from M51)
      -> archive officer creates ArchiveCollection per content_type
      -> IngestJob triggered per collection:
            -> source data queried from owning module
            -> BagIt bag assembled:
                  -> payload files written to data/
                  -> manifest-sha256.txt generated
                  -> bag-info.txt populated (Bagging-Date, Bag-Size, Bag-Oxum)
                  -> tagmanifest-sha256.txt generated
            -> bag uploaded to MinIO (archive bucket, retention lock)
            -> IngestJob status = COMPLETE
            -> ArchiveCollection status = INGESTED
      -> for RESULTS and REPRODUCIBILITY_BUNDLE collections:
            -> DOI minted via DataCite API (Schema v4.4)
            -> doi stored in ArchiveCollection
            -> ArchiveCollection status = DOI_MINTED
            -> emit eel.archive.doi_minted.v1
      -> quarterly PreservationAttestation:
            -> re-download bag from MinIO
            -> re-compute SHA-256; compare to bag_checksum
            -> re-verify bag-oxum
            -> PRONOM format check: set format_migration_needed if needed
            -> officer signs attestation with HSM
            -> emit eel.archive.preservation_attested.v1
      -> at retention_expiry (or legal order):
            -> commissioner approves DestructionCertificate (second officer witness)
            -> MinIO object deletion (retention lock lifted by authorised admin)
            -> ArchiveCollection status = DESTROYED
            -> emit eel.archive.collection_destroyed.v1
      -> for permanent-retention collections:
            -> KNADSHandover created; physical media or SFTP transfer to KNADS
            -> KNADS acknowledgement recorded
            -> ArchiveCollection status = KNADS_TRANSFERRED
```

## 5. API surface

Contract: `packages/openapi-contracts/support-admin/archive.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/archive/collections` | Archive officer (JWT + SpiceDB `ARCHIVE_OFFICER`) | Create archive collection |
| `GET /v1/archive/collections/{id}` | Archive officer, OAG auditor | Collection status + DOI |
| `GET /v1/archive/collections/{id}/bag` | Archive officer | Pre-signed MinIO URL for bag download |
| `POST /v1/archive/collections/{id}/attest` | Archive officer + HSM | Create preservation attestation |
| `POST /v1/archive/collections/{id}/destroy` | Commissioner + HSM | Create destruction certificate |
| `POST /v1/archive/collections/{id}/knads-handover` | Archive officer | Initiate KNADS handover |
| `GET /v1/archive/collections` | Archive officer | List collections by election cycle + status |
| `GET /v1/archive/doi/{doi}` | Public | Resolve DOI to DIP download (rate-limited 20/hr) |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Archive management console (Django) | `services/support-admin-svc/django/archive_admin` | Create collections, manage attestations, KNADS handover |
| Preservation dashboard (LiveView) | `services/support-admin-svc/ex/archive_web` | Quarterly attestation queue; format migration alerts |
| Public DOI resolution (Astro) | `services/portal-svc/astro/doi-resolver` | Public download of DIP for citable collections |

Languages: EN. WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **MinIO retention lock**: bags are stored with an S3 object lock (GOVERNANCE mode, minimum retention = `retention_years`); they cannot be deleted without the `s3:BypassGovernanceRetention` IAM permission, which is restricted to the archive admin role.
- **Bag checksum chain**: `ArchiveCollection.bag_checksum` is the SHA-256 of the entire bag directory tree (not just the payload); the quarterly attestation re-verifies this checksum. A mismatch triggers a `CORRUPTION_ALERT` and the collection is quarantined.
- **BagIt Oxum verification**: `Bag-Oxum` (RFC 8493 §2.2.3) records payload octet count and file count; re-verification on every attestation detects partial file deletions.
- **DestructionCertificate witness**: a second officer must witness destruction; the `destruction_witness` field is a distinct officer UUID from `approved_by_commissioner`. Postgres constraint enforces this.
- **DOI immutability**: once minted, DOI metadata is versioned (DataCite API allows metadata update but retains version history); breaking metadata changes require a new DOI version, not a silent overwrite.
- **Audit shadow**: all archive mutations logged to `eel_admin_audit_shadow` with officer identity and timestamp.

## 8. Kenya statute alignment

- **Public Archives and Documentation Service Act Cap 19 §9 (transfer to KNADS)**: election records are public records; permanent-retention collections must be transferred to KNADS at the end of their active management period.
- **Elections Act §6B (records retention)**: electoral commission records must be retained for at least 7 years; `retention_years` is set to 7 by default for financial and training records, and indefinitely for results and audit trails.
- **Kenya Data Protection Act §39 (retention limitation)**: records containing personal data must be destroyed when retention period expires; M69's `DestructionCertificate` is the ODPC-auditable destruction record.
- **Statistics Act 2006 §25 (publication of official statistics)**: DOI-minted election result datasets are citable public statistics; DataCite DOI registration is the mechanism for official citation.

## 9. Honest caveats + external prerequisites

- DataCite DOI minting requires the Commission to be a registered DataCite member or to obtain DOI allocation through a Kenyan DataCite consortium member (e.g., a Kenyan research institution). The DataCite API adapter is ready; membership activation is a procurement task.
- KNADS digital transfer infrastructure depends on KNADS having an SFTP endpoint or API; M69 supports physical media transfer as a fallback. Physical media transfer requires courier logistics outside IEMS.
- Format migration (when `format_migration_needed = true`) is a manual task: the archive officer downloads the bag, migrates formats using the recommended tool, and re-ingests. M69 tracks the migration decision but does not automate format conversion.
- MinIO GOVERNANCE retention lock can be bypassed by an admin with `s3:BypassGovernanceRetention`. COMPLIANCE mode (immutable) would provide stronger guarantees but does not support early destruction for legal orders. The GOVERNANCE choice is a deliberate trade-off.
- The PRONOM format check uses the UK National Archives' PRONOM registry; its relevance to Kenyan archival practice depends on KNADS adopting PRONOM identifiers. If KNADS uses a different format registry, the format migration logic may need updating.

## 10. Test coverage + operational runbooks

Coverage under `services/support-admin-svc/crates/archive/tests/`: BagIt bag assembly and Oxum verification (property-based, fuzz payload file sets), SHA-256 checksum round-trip (property-based), DataCite metadata schema validation (golden-file against v4.4 schema), DestructionCertificate witness constraint (integration: same officer as approver must fail), MinIO retention lock (integration with MinIO testcontainer), KNADS handover state machine (integration).

Runbooks under `docs/runbooks/`:

- `support-admin-svc.md` (section: long-term-archive) -- deploy, rollback, bag repair.
- `bag-corruption-response.md` -- quarantine and re-ingest procedure for a corrupted bag.
- `knads-handover-procedure.md` -- step-by-step KNADS digital and physical transfer.
- `doi-minting-failure.md` -- DataCite API failure and manual DOI registration fallback.

Related documents: full spec at `docs/plans/modules/long-term-archive.md`; service capability catalogue at `services/support-admin-svc/docs/CAPABILITIES.md`; M14 reproducibility bundle at `docs/features/reproducibility-bundle.md`.

### Retention schedule

| Content type | Retention years | Basis |
|---|---|---|
| RESULTS | Permanent (KNADS transfer) | Elections Act §6B + Public Archives Cap 19 |
| AUDIT_TRAIL | Permanent (KNADS transfer) | Elections Act §6B |
| VOTER_REGISTER | 10 years then reassess | Kenya Data Protection Act §39 |
| FINANCIAL | 7 years | PFM Act §91 |
| LEGAL | Permanent (KNADS transfer) | Public Archives Cap 19 |
| REPRODUCIBILITY_BUNDLE | Permanent (KNADS transfer) | Elections Act §6B |
| TRAINING_RECORDS | 7 years | Employment Act + Elections Act |

KNADS-transfer collections have `retention_years = NULL` and `retention_expiry = NULL`; they remain in `PRESERVED` state until the KNADS handover is completed, after which they transition to `KNADS_TRANSFERRED`.

### BagIt bag structure (RFC 8493)

```
election-2027-ke-results/
  bagit.txt                     # BagIt version + encoding declaration
  bag-info.txt                  # Bagging-Date, Bag-Size, Bag-Oxum, Source-Organization
  manifest-sha256.txt           # SHA-256 checksum per payload file
  tagmanifest-sha256.txt        # SHA-256 checksum of tag files
  data/
    results-certified.parquet   # M51 certified results
    merkle-proof.json           # P05 transparency log proof
    form34a-images/             # Scanned form images (optional)
    metadata.json               # DataCite v4.4 metadata record
```
