# Data Model — IEBC IEMS (All 69 Modules)

**Companion to:** `PRD.md`, `ARCHITECTURE.md`, `API-CATALOG.md`.
**Version:** 2.0 — Full 69-module canonical reference.
**Status:** Pre-award bid specification; DDL and event schemas delivered at project execution.

Conventions:
- All IDs are UUID7 strings (`uuid7str()` from `uuid6` PyPI package).
- Timestamps stored UTC (ISO 8601), presented EAT (UTC+3) in UIs.
- Monetary values are integer **cents** (KES), never floats.
- Biometric templates are field-level encrypted (AES-256-GCM) with keys separate from index rows, managed by HSM.
- `PII: HIGH` = direct personal identifier; `PII: MED` = indirect/derived; `PII: NONE` = no personal data.

---

## 1. Executive Summary

The IEBC IEMS data model covers **218 distinct entity types** across 12 bounded contexts, serving 69 software modules and the platform layer. The decision to build an **event-sourced core** (per `DATACRAFT_SOFTWARE_SCOPE.md` §0.4) means every critical state change is an immutable, hash-chained event appended to an append-only store. State views are derived by replaying events through deterministic projections. This architecture provides complete tamper-evident audit trails and allows deterministic re-derivation of any election result from signed source data.

Single-source-of-truth policy: each entity has exactly one **owner module** responsible for its schema, lifecycle, and migration. All other modules that read or reference that entity do so through the owner's API or a well-defined read-model projection. Cross-context references use UUID7 foreign keys; no embedded documents span context boundaries. Reference data (boundaries, statutory codes) is owned by the Electoral Administration context and treated as effectively immutable once gazetted; changes require dual-commissioner approval and a new version record.

Sensitive PII (biographic fields, biometric templates, ID scans) is field-encrypted at write time. Keys are held in the HSM cluster and never leave Kenya. The escrow mirror receives only AuditEvents, MerkleLogRoots, and ResultsPublicationSnapshots — never voter biometrics or PII. Elections Act evidentiary retention applies to results data (permanent), voter register (contract + 5 years), and audit logs (15 years per `expect.md`). DPA 2019 §26 lawful basis for biometric processing is explicit consent at registration.

---

## 2. Domain-Driven Design Boundaries

| Bounded Context | Modules | Core Aggregates | Notes |
|---|---|---|---|
| **VoterRegistration** | M01, M02, M03, M04, M05, M06, M07, M35, M36, M53, M64 | Voter, BiometricTemplate, VoterRegisterEvent | Event-sourced; biometrics field-encrypted |
| **CandidateParty** | M12, M22, M45, M46, M47, M67 | Candidate, PoliticalParty, NominationFeePayment | Dependent on VoterRegistration for voter_id FK |
| **ElectionAdmin** | M48, M49, M50, M51, M52, M54, M58 | Election, ElectivePosition, GazetteNotice, Declaration | Authoritative for election schedule |
| **BallotResults** | M08, M09, M10, M11, M14, M21, M39, M63 | Form34A-C, ResultsPublicationSnapshot, ReproducibilityBundle | Merkle-anchored, immutable once signed |
| **KitInventory** | M20, M32, M33 | Kit, Asset, CustodyEvent, Facility | Physical asset lifecycle |
| **PollingOperations** | M05, M06, M19, M24, M25, M27, M31 | PollingSession, VoterIdentificationEvent, QueueSnapshot | Event-sourced identification stream |
| **AuditCompliance** | M15, M21, M28, M59, M61, M62, M69 | AuditEvent, MerkleLogRoot, EvidenceBundle | Append-only; 15-year retention |
| **FieldOps** | M16, M17, M23, M26, M29, M30, M34, M37, M38, M40, M41, M44, M60 | GrievanceCase, ECCAlert, DetectionAlert | Operational situational awareness |
| **AccreditationObservers** | M42, M43, M57 | MediaAccreditation, ObserverOrganisation, ObserverMission | Read-only after badge issue |
| **ElectoralBoundaries** | M18, M56 | Boundary, County, Constituency, Ward, PollingStation | Immutable once gazetted |
| **Finance** | M22, M47, M54, M55, M66 | ElectionBudget, NominationFeePayment, PayrollRun | Audit-grade financial records |
| **IdentityAccess** | P04 (IAM/PKI) | User, Role, UserRoleAssignment, KitCertificate | Cross-cutting; consumed by all contexts |

Anti-corruption layers between contexts:
- VoterRegistration → CandidateParty: read-only projection of Voter (voter_id, status, name, constituency). No write path from CRMS into voter register.
- BallotResults → VoterRegistration: read-only voter count snapshot at poll-open; no voter_id attached to any ballot or result (ballot secrecy per Constitution Art. 138(2)).
- ElectoralBoundaries → all: boundary data is cached as read-model; updates propagate via Kafka `boundary.updated` events.

---

## 3. Entity Catalogue

### 3.1 VoterRegistration Context

#### Voter
| Field | Type | Req | Notes |
|---|---|---|---|
| voter_id | str (UUID7) | Y | Immutable primary key |
| id_number | str(20) | Y | Kenya national ID / Maisha / Passport — encrypted at rest |
| id_type | enum | Y | `2ND_GEN_ID`, `MAISHA_CARD`, `PASSPORT` |
| id_expiry | date | N | Passport only |
| first_name | str(100) | Y | Alpha-only validated |
| middle_name | str(100) | N | |
| last_name | str(100) | Y | Alpha-only validated |
| date_of_birth | date | Y | Must be ≥ 18 at registration date |
| gender | enum | Y | `MALE`, `FEMALE`, `INTERSEX` |
| mobile | str(15) | N | E.164; encrypted |
| email | str(320) | N | Encrypted |
| physical_challenge_type | enum | Y | `NONE`, `VISUAL`, `HEARING`, `PHYSICAL`, `INTELLECTUAL`, `OTHER` |
| assistance_required | bool | Y | |
| education_level | enum | Y | `NONE`, `PRIMARY`, `SECONDARY`, `TERTIARY`, `UNIVERSITY`, `POSTGRAD` |
| status | enum | Y | `ACTIVE`, `DECEASED`, `DELETED`, `SUSPENDED`, `TRANSFERRED`, `DORMANCY_PENDING`, `REMOVAL_PROPOSED`, `REMOVAL_CONFIRMED` |
| registration_polling_station_id | str (UUID7) | Y | FK → PollingStation |
| current_polling_station_id | str (UUID7) | Y | Updated on transfer |
| registration_date | datetime | Y | |
| gazetted_cutoff_date | datetime | N | Frozen for specific election |
| kit_id_at_registration | str (UUID7) | Y | FK → Kit |
| enrolment_clerk_id | str (UUID7) | Y | FK → User |
| registration_signature_hash | str(64) | Y | SHA-256 of signed record |
| version | int | Y | Optimistic concurrency counter |

**Relationships:** belongs to PollingStation (registration); owned by Kit (captured by); enrolled by User.
**Owner module:** M02 (BVR Backend). **Referenced by:** M01, M03, M04, M05, M06, M07, M12, M13, M35, M36, M53, M64.
**Lifecycle:** versioned-mutable (status changes write new version; biometrics immutable).
**Retention:** contract + 5 years (Elections Act §44; DPA 2019 §25).
**PII:** HIGH. **Index:** `id_number` (unique, encrypted), `polling_station_id`, `status`, `registration_date`.

---

#### BiometricTemplate
| Field | Type | Req | Notes |
|---|---|---|---|
| template_id | str (UUID7) | Y | |
| voter_id | str (UUID7) | Y | FK → Voter |
| modality | enum | Y | `FINGERPRINT`, `IRIS`, `FACIAL` |
| finger_position | int | N | 1–10 per ANSI/NIST; null for iris/face |
| eye | enum | N | `LEFT`, `RIGHT` (iris only) |
| format | enum | Y | `ISO_19794_2`, `WSQ`, `JPEG_2000`, `ISO_19794_5`, `ICAO_9303`, `ISO_19794_6` |
| template_encrypted | bytes | Y | AES-256-GCM; key in HSM |
| template_key_id | str (UUID7) | Y | HSM-wrapped key reference |
| quality_score | int | Y | 0–100 (NIST NFIQ 2.0 or equivalent) |
| captured_at | datetime | Y | |
| captured_by | str (UUID7) | Y | FK → User |
| captured_by_kit_id | str (UUID7) | Y | FK → Kit |
| ansi_nist_record | bytes | Y | ANSI/NIST-ITL 1-2011 wrapped form |

**Owner:** M02. **Lifecycle:** immutable once stored. **Retention:** same as Voter.
**PII:** HIGH (biometric). **Index:** `voter_id`, `modality`. Templates never leave the biometric context unencrypted.

---

#### IDDocumentScan
| Field | Type | Req | Notes |
|---|---|---|---|
| scan_id | str (UUID7) | Y | |
| voter_id | str (UUID7) | Y | FK → Voter |
| document_type | enum | Y | `2ND_GEN_ID`, `MAISHA_CARD`, `PASSPORT` |
| pdf_encrypted | bytes | Y | Encrypted PDF from device scanner |
| mrz_parsed | jsonb | Y | Parsed MRZ fields |
| captured_at | datetime | Y | |
| kit_id | str (UUID7) | Y | FK → Kit |

**Owner:** M01/M02. **Lifecycle:** immutable. **Retention:** same as Voter. **PII:** HIGH.

---

#### VoterSignature
| Field | Type | Req | Notes |
|---|---|---|---|
| signature_id | str (UUID7) | Y | |
| voter_id | str (UUID7) | Y | FK → Voter |
| signature_image | bytes | Y | On-device stylus capture |
| captured_at | datetime | Y | |
| kit_id | str (UUID7) | Y | FK → Kit |

**Owner:** M01/M02. **Lifecycle:** immutable. **Retention:** same as Voter. **PII:** HIGH.

---

#### VPRSApplication
| Field | Type | Req | Notes |
|---|---|---|---|
| application_id | str (UUID7) | Y | |
| id_number | str(20) | Y | Encrypted |
| id_type | enum | Y | |
| applicant_data | jsonb | Y | Biographic form |
| iprs_verified | bool | Y | |
| iprs_verification_at | datetime | N | |
| preferred_polling_station_id | str (UUID7) | N | FK → PollingStation |
| status | enum | Y | `DRAFT`, `SUBMITTED`, `VERIFIED`, `CONVERTED_TO_VOTER`, `REJECTED`, `EXPIRED` |
| qr_handoff_token | str(64) | N | For KIEMS device pick-up |
| qr_expires_at | datetime | N | |
| converted_voter_id | str (UUID7) | N | FK → Voter on conversion |

**Owner:** M04. **Lifecycle:** soft-delete. **Retention:** 12 months, then anonymised. **PII:** HIGH.

---

#### VoterRegisterEvent
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| voter_id | str (UUID7) | Y | FK → Voter |
| event_type | enum | Y | `DEATH_PROPOSED`, `DEATH_CONFIRMED`, `ID_CORRECTION`, `DORMANCY_FLAG`, `FAMILY_NOTIFICATION`, `REMOVAL_APPROVED`, `REMOVAL_REJECTED`, `REINSTATEMENT` |
| source | enum | Y | `CRS`, `NRB`, `KRA`, `FAMILY_SHORTCODE`, `OFFICER_MANUAL` |
| confidence_score | float | N | 0.0–1.0 for automated matches |
| batch_id | str | N | FK → BatchIngestionReport |
| officer_id | str | N | FK → User |
| decision | enum | N | `APPROVED`, `REJECTED`, `PENDING`, `HELD` |
| decision_basis | str | N | Statutory code + free text |
| created_at | datetime | Y | |
| resolved_at | datetime | N | |

**Owner:** M35. **Lifecycle:** append-only event. **Retention:** permanent. **PII:** MED.
**Causal predecessor:** source system batch or family notification. **Downstream:** M36 DeregistrationCase.

---

#### RegisterSnapshot
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| snapshot_at | datetime | Y | |
| total_registered | int | Y | |
| root_hash | str(64) | Y | SHA-256 of sorted voter_id list |
| merkle_root | str(64) | Y | Merkle root of all active voter records |
| gazette_notice_id | str | N | FK → GazetteNotice |
| signed_by | str | Y | FK → User (IEBC Commissioner) |
| signed_at | datetime | N | |

**Owner:** M35. **Lifecycle:** immutable once signed. **Retention:** permanent. **PII:** NONE.

---

#### BatchIngestionReport
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| source | enum | Y | `CRS`, `NRB`, `KRA` |
| received_at | datetime | Y | |
| record_count | int | Y | |
| matched_count | int | Y | |
| unmatched_count | int | Y | |
| error_count | int | Y | |
| batch_hash | str(64) | Y | SHA-256 of incoming batch file |

**Owner:** M35. **Lifecycle:** immutable. **Retention:** 15 years. **PII:** NONE.

---

#### DeregistrationCase
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| voter_id | str (UUID7) | Y | FK → Voter |
| m35_event_id | str | Y | Source VoterRegisterEvent |
| status | enum | Y | `NOTICED`, `APPEAL_FILED`, `UNDER_REVIEW`, `ADJUDICATION`, `OVERTURNED`, `CONFIRMED`, `DEREGISTERED`, `EXPIRED_NO_APPEAL` |
| notice_dispatched_at | datetime | N | |
| notice_deadline | datetime | Y | `notice_dispatched_at + 30 days` |
| appeal_id | str | N | FK → AppealRecord |
| officer_id | str | N | FK → User |

**Owner:** M36. **Lifecycle:** mutable until terminal state. **Retention:** contract + 5 years. **PII:** MED.

---

#### AppealRecord
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| deregistration_case_id | str | Y | FK → DeregistrationCase |
| appellant_id_hash | str(64) | Y | SHA-256 of national ID |
| grounds | str | Y | Stated grounds for appeal |
| evidence_file_ids | list[str] | Y | Encrypted blob references |
| submitted_at | datetime | Y | |
| hearing_date | datetime | N | |
| adjudicator_id | str | N | FK → User |
| outcome | enum | N | `UPHELD`, `DISMISSED` |
| outcome_at | datetime | N | |

**Owner:** M36. **Lifecycle:** immutable once decided. **Retention:** 15 years. **PII:** MED.

---

#### VoterCard
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| voter_id | str (UUID7) | Y | FK → Voter |
| batch_id | str | Y | FK → CardProductionBatch |
| card_serial | str(20) | Y | Unique per card |
| qr_payload | str | Y | Encrypted JSON: voter_id + constituency |
| status | enum | Y | `ORDERED`, `PRINTED`, `DISPATCHED`, `DELIVERED`, `ACTIVATED`, `REPLACED`, `REVOKED` |
| issued_at | datetime | N | |
| collected_at | datetime | N | |

**Owner:** M53. **Lifecycle:** mutable. **Retention:** contract + 5 years. **PII:** MED.

---

#### CardProductionBatch
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| order_date | date | Y | |
| quantity | int | Y | |
| vendor_ref | str | Y | Al Ghurair or equivalent |
| delivery_date | date | N | |
| status | enum | Y | `ORDERED`, `IN_PRODUCTION`, `SHIPPED`, `RECEIVED`, `DISTRIBUTED` |

**Owner:** M53. **Lifecycle:** mutable. **Retention:** permanent. **PII:** NONE.

---

#### RollVersion
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| constituency_code | str | Y | |
| version_number | int | Y | Monotonic |
| published_at | datetime | Y | |
| root_hash | str(64) | Y | Merkle root of version |
| total_count | int | Y | |
| signed_by | str | Y | FK → User |

**Owner:** M64. **Lifecycle:** immutable once published. **Retention:** permanent. **PII:** NONE.

---

#### Objection
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| roll_version_id | str | Y | FK → RollVersion |
| objector_id_hash | str(64) | Y | Hashed ID of person filing |
| target_voter_id | str | N | Voter being challenged |
| grounds | str | Y | |
| status | enum | Y | `FILED`, `ACKNOWLEDGED`, `HEARING_SCHEDULED`, `UPHELD`, `DISMISSED` |
| filed_at | datetime | Y | |

**Owner:** M64. **Retention:** 15 years. **PII:** MED.

---

### 3.2 ElectoralBoundaries Context

#### County
| Field | Type | Req | Notes |
|---|---|---|---|
| county_id | str (UUID7) | Y | |
| code | str(3) | Y | Statutory IEBC code (001–047) |
| name | str(100) | Y | |
| region | enum | Y | |
| county_returning_officer_id | str (UUID7) | N | FK → User |

**Owner:** M18. **Lifecycle:** immutable. **Retention:** permanent. **PII:** NONE.
**Index:** `code` (unique).

---

#### Constituency
| Field | Type | Req | Notes |
|---|---|---|---|
| constituency_id | str (UUID7) | Y | |
| code | str(6) | Y | Statutory (290 total) |
| name | str(100) | Y | |
| county_id | str (UUID7) | Y | FK → County |
| returning_officer_id | str (UUID7) | N | FK → User |

**Owner:** M18. **Lifecycle:** immutable. **Retention:** permanent. **PII:** NONE.

---

#### Ward
| Field | Type | Req | Notes |
|---|---|---|---|
| ward_id | str (UUID7) | Y | |
| code | str(8) | Y | |
| name | str(100) | Y | |
| constituency_id | str (UUID7) | Y | FK → Constituency |

**Owner:** M18. **Lifecycle:** immutable. **Retention:** permanent. **PII:** NONE.

---

#### PollingCentre
| Field | Type | Req | Notes |
|---|---|---|---|
| polling_centre_id | str (UUID7) | Y | |
| code | str(10) | Y | |
| name | str(200) | Y | |
| ward_id | str (UUID7) | Y | FK → Ward |
| gps_latitude | decimal(9,6) | Y | |
| gps_longitude | decimal(9,6) | Y | |
| address | str | Y | |

**Owner:** M18. **Lifecycle:** versioned. **Retention:** permanent. **PII:** NONE.

---

#### PollingStation
| Field | Type | Req | Notes |
|---|---|---|---|
| polling_station_id | str (UUID7) | Y | |
| code | str(12) | Y | Statutory code |
| name | str(200) | Y | e.g. "STREAM 1" |
| polling_centre_id | str (UUID7) | Y | FK → PollingCentre |
| max_voters | int | Y | Default 700 |
| geo_fence_polygon | geography | Y | PostGIS polygon |
| presiding_officer_id | str (UUID7) | N | FK → User (per election) |
| kit_id | str (UUID7) | N | FK → Kit (per election) |
| qr_code_hash | str(64) | N | Signed QR for kit-station pairing |

**Owner:** M18. **Lifecycle:** versioned per election. **Retention:** permanent. **PII:** NONE.
**Index:** `code` (unique), `polling_centre_id`, `geo_fence_polygon` (GiST).

---

#### Boundary
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| code | str | Y | Official IEBC code |
| name | str | Y | |
| boundary_type | enum | Y | `CONSTITUENCY`, `WARD`, `COUNTY`, `STATION` |
| parent_code | str | N | |
| geojson | jsonb | Y | GeoJSON Feature |
| population_2019 | int | N | |
| registered_voters | int | Y | Live, updated per CVR cycle |
| area_km2 | float | Y | |
| effective_date | datetime | Y | |
| gazette_notice_ref | str | Y | |
| geojson_hash | str(64) | Y | SHA-256 of canonical GeoJSON |
| version | int | Y | |
| signed_by | str | Y | FK → User |

**Owner:** M18. **Lifecycle:** versioned-mutable. **Retention:** permanent. **PII:** NONE.
**Index:** `code`, `boundary_type`, geospatial index on GeoJSON geometry.

---

### 3.3 ElectionAdmin Context

#### Election
| Field | Type | Req | Notes |
|---|---|---|---|
| election_id | str (UUID7) | Y | |
| type | enum | Y | `GENERAL`, `BY_ELECTION`, `REFERENDUM`, `PRESIDENTIAL_RERUN` |
| gazette_date | date | Y | |
| poll_date | date | Y | |
| registration_cutoff | datetime | Y | |
| poll_open_time | time | Y | |
| poll_close_time | time | Y | |
| status | enum | Y | `PLANNED`, `REGISTRATION_OPEN`, `REGISTRATION_CLOSED`, `PRE_POLL`, `POLL_OPEN`, `POLL_CLOSED`, `COUNTING`, `RESULTS_PROVISIONAL`, `RESULTS_FINAL` |

**Owner:** M52. **Lifecycle:** mutable until `RESULTS_FINAL`. **Retention:** permanent. **PII:** NONE.

---

#### ElectivePosition
| Field | Type | Req | Notes |
|---|---|---|---|
| position_id | str (UUID7) | Y | |
| election_id | str (UUID7) | Y | FK → Election |
| type | enum | Y | `PRESIDENT`, `GOVERNOR`, `SENATOR`, `MP`, `CWR`, `MCA` |
| scope | enum | Y | `NATIONAL`, `COUNTY`, `CONSTITUENCY`, `WARD` |
| scope_id | str (UUID7) | Y | FK to relevant area entity |
| form_template | enum | Y | `FORM_34A`, `FORM_35A`, `FORM_36A`, `FORM_37A`, `FORM_38A` |

**Owner:** M52. **Lifecycle:** immutable once gazetted. **Retention:** permanent. **PII:** NONE.

---

#### ElectionEvent
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| election_id | str | Y | FK → Election |
| event_type | enum | Y | `NOMINATION_OPEN`, `NOMINATION_CLOSE`, `CAMPAIGN_OPEN`, `POLL_DAY`, `RESULTS_DAY`, `PETITION_DEADLINE` |
| scheduled_at | datetime | Y | |
| gazette_notice_id | str | N | FK → GazetteNotice |
| actual_at | datetime | N | |

**Owner:** M52. **Lifecycle:** immutable. **Retention:** permanent. **PII:** NONE.

---

#### GazetteNotice
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| notice_type | enum | Y | `ELECTION_DATE`, `NOMINATION`, `CANDIDATE_LIST`, `REGISTER_CLOSE`, `RESULT`, `RECALL`, `REFERENDUM`, `BOUNDARY` |
| gazette_number | str | Y | Kenya Gazette reference |
| publication_date | date | Y | |
| content_hash | str(64) | Y | SHA-256 of gazette text |
| supplement_url | str | N | Link to Kenya Gazette Online |
| election_id | str | N | FK → Election |

**Owner:** M52. **Lifecycle:** immutable. **Retention:** permanent. **PII:** NONE.

---

#### ByElection
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| seat_type | enum | Y | `MP`, `SENATOR`, `GOVERNOR`, `CWR`, `MCA`, `PRESIDENTIAL` |
| constituency_code | str | Y | |
| trigger_reason | enum | Y | `DEATH`, `RECALL`, `NULLIFICATION`, `VACANCY` |
| vacancy_date | date | Y | |
| polling_date | date | Y | |
| status | enum | Y | `DRAFT`, `GAZETTED`, `NOMINATIONS_OPEN`, `POLLS_OPEN`, `DECLARED`, `CERTIFIED` |

**Owner:** M50. **Lifecycle:** mutable. **Retention:** permanent. **PII:** NONE.

---

#### Referendum
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| title_en | str | Y | |
| title_sw | str | Y | |
| question_en | str | Y | |
| question_sw | str | Y | |
| pathway | enum | Y | `ART255`, `ART256`, `ART257`, `POPULAR_INITIATIVE` |
| voting_date | date | Y | |
| county_threshold | int | Y | 24 for constitutional; 0 if not applicable |
| status | enum | Y | `DRAFT`, `CONFIGURED`, `ACTIVE`, `CLOSED`, `DECLARED`, `PETITIONED` |

**Owner:** M49. **Lifecycle:** mutable. **Retention:** permanent. **PII:** NONE.

---

#### Declaration
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| election_event_id | str | Y | FK → ElectionEvent |
| seat_type | enum | Y | |
| constituency_code | str | Y | |
| declared_winner_candidate_id | str | N | FK → Candidate; None for referendum |
| declared_by | str | Y | FK → User (RO) |
| declared_at | datetime | Y | |
| declaration_statement_hash | str(64) | Y | SHA-256 of signed PDF |
| certificate_of_election_id | str | N | FK → CertificateOfElection |
| gazette_notice_id | str | N | FK → GazetteNotice |
| petition_window_closes | date | Y | |
| petition_lodged | bool | Y | |
| status | enum | Y | `DRAFT`, `LEGAL_REVIEW`, `DECLARED`, `CERTIFIED`, `PETITIONED` |
| merkle_inclusion_proof | str | N | |

**Owner:** M51. **Lifecycle:** immutable once declared. **Retention:** permanent. **PII:** NONE.

---

#### CertificateOfElection
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| declaration_id | str | Y | FK → Declaration |
| candidate_id | str | Y | FK → Candidate |
| candidate_name_en | str | Y | |
| seat | str | Y | |
| constituency | str | Y | |
| election_date | date | Y | |
| votes_received | int | Y | |
| total_valid_votes | int | Y | |
| ro_signature_hash | str(64) | Y | |
| iebc_seal_hash | str(64) | Y | |

**Owner:** M51. **Lifecycle:** immutable. **Retention:** permanent. **PII:** MED (candidate name).

---

### 3.4 CandidateParty Context

#### Candidate
| Field | Type | Req | Notes |
|---|---|---|---|
| candidate_id | str (UUID7) | Y | |
| election_id | str (UUID7) | Y | FK → Election |
| position_id | str (UUID7) | Y | FK → ElectivePosition |
| voter_id | str (UUID7) | Y | FK → Voter (must be registered voter) |
| party_id | str (UUID7) | N | FK → PoliticalParty; null if independent |
| deputy_id | str (UUID7) | N | FK → Candidate (President/Governor tickets) |
| portrait | bytes | Y | ICAO-compliant photo |
| ballot_symbol | bytes | N | Party symbol or independent symbol |
| ballot_order | int | N | Assigned at nomination close |
| clearance_status | enum | Y | `SUBMITTED`, `DOCS_UPLOADED`, `RO_REVIEWING`, `CLEARED`, `REJECTED`, `GAZETTED` |

**Owner:** M12. **Lifecycle:** mutable until gazetted, immutable after. **Retention:** contract + 5 years. **PII:** MED.

---

#### PoliticalParty
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| name_en | str | Y | |
| name_sw | str | N | |
| abbreviation | str | Y | |
| registration_number | str | N | ORPP number |
| ideology | str | Y | |
| status | enum | Y | `DRAFT`, `SUBMITTED`, `APPROVED`, `CONDITIONAL`, `REJECTED`, `GAZETTED`, `SUSPENDED`, `DEREGISTERED` |
| symbol_registry_id | str | N | FK → SymbolRegistration |
| colour_hex | str | Y | |
| verified_membership_count | int | Y | |
| compliance_score | int | Y | 0–100 |
| coalition_id | str | N | FK → CoalitionRegistration |

**Owner:** M45 (extends M12 Party). **Lifecycle:** mutable. **Retention:** permanent. **PII:** NONE.

---

#### CoalitionRegistration
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| name | str | Y | |
| member_party_ids | list[str] | Y | FK list → PoliticalParty |
| mou_signed_at | datetime | Y | |
| effective_from | date | Y | |
| effective_to | date | N | |
| status | enum | Y | `DRAFT`, `RATIFIED`, `DISSOLVED` |

**Owner:** M45. **Retention:** permanent. **PII:** NONE.

---

#### SymbolRegistration
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| party_id | str | Y | FK → PoliticalParty |
| symbol_image_hash | str(64) | Y | SHA-256 of approved symbol image |
| colour_palette | list[str] | Y | Hex codes |
| orpp_approval_ref | str | Y | |
| effective_date | date | Y | |
| status | enum | Y | `PENDING`, `APPROVED`, `REVOKED` |

**Owner:** M67. **Lifecycle:** immutable once approved. **Retention:** permanent. **PII:** NONE.

---

#### NominationFeePayment
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| candidate_id | str | Y | FK → Candidate |
| fee_schedule_id | str | Y | FK → FeeSchedule |
| amount_kes_cents | int | Y | Integer cents |
| payment_provider | enum | Y | `MPESA`, `AIRTEL_MONEY`, `BANK_TRANSFER` |
| provider_ref | str | N | Provider transaction ID |
| status | enum | Y | `PENDING`, `CONFIRMED`, `FAILED`, `EXPIRED`, `WAIVED`, `REFUNDED` |
| paid_at | datetime | N | |
| waiver_id | str | N | FK → WaiverApplication |

**Owner:** M47. **Lifecycle:** immutable once confirmed. **Retention:** 15 years. **PII:** NONE.

---

#### PrimaryOptIn
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| party_id | str | Y | FK → PoliticalParty |
| election_cycle | str | Y | |
| mou_signed_at | datetime | Y | |
| approved_by | str | Y | FK → User |

**Owner:** M46. **Lifecycle:** immutable. **Retention:** 15 years. **PII:** NONE.

---

#### RecallPetition
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| target_candidate_id | str | Y | FK → Candidate |
| grounds | enum | Y | `CONVICTION`, `CHAPTER_SIX`, `ABUSE_OF_OFFICE` |
| constituency_code | str | Y | |
| signature_target | int | Y | 30% of registered voters per Art. 104 |
| signatures_verified | int | Y | |
| status | enum | Y | `SUBMITTED`, `SIGNATURE_VERIFICATION`, `THRESHOLD_MET`, `THRESHOLD_NOT_MET`, `ELECTION_DECLARED` |
| initiated_at | datetime | Y | |

**Owner:** M48. **Lifecycle:** mutable until terminal. **Retention:** 15 years. **PII:** MED.

---

### 3.5 BallotResults Context

#### PollingSession
| Field | Type | Req | Notes |
|---|---|---|---|
| session_id | str (UUID7) | Y | |
| polling_station_id | str (UUID7) | Y | FK → PollingStation |
| election_id | str (UUID7) | Y | FK → Election |
| kit_id | str (UUID7) | Y | FK → Kit |
| presiding_officer_id | str (UUID7) | Y | FK → User |
| opened_at | datetime | Y | |
| opened_signature | str(64) | Y | HSM-countersigned |
| closed_at | datetime | N | |
| closed_reason | enum | N | `NORMAL`, `EMERGENCY_CLOSE` (Elections Act §55) |
| voters_registered | int | Y | Snapshot at open |
| voters_authenticated | int | Y | Live counter |

**Owner:** M06. **Lifecycle:** immutable once closed. **Retention:** permanent. **PII:** NONE.

---

#### VoterIdentificationEvent
| Field | Type | Req | Notes |
|---|---|---|---|
| event_id | str (UUID7) | Y | |
| session_id | str (UUID7) | Y | FK → PollingSession |
| voter_id | str (UUID7) | Y | FK → Voter |
| modality | enum | Y | `FINGERPRINT`, `IRIS`, `FACIAL`, `MRZ_FALLBACK` |
| result | enum | Y | `AUTHENTICATED`, `NOT_FOUND`, `REDIRECTED`, `DISTRESS` |
| timestamp | datetime | Y | |
| kit_id | str (UUID7) | Y | FK → Kit |
| gnss_fix | geography | N | Point |
| signed_event | bytes | Y | HSM-signed |

**Owner:** M06. **Lifecycle:** immutable, event-sourced. **Retention:** permanent. **PII:** MED (voter_id + session).
**Note:** biometric templates NOT retransmitted; only voter_id and outcome.
**Downstream projections:** turnout counter, anomaly detection (M26), audit (P05).

---

#### Form34A
| Field | Type | Req | Notes |
|---|---|---|---|
| form_id | str (UUID7) | Y | |
| election_id | str (UUID7) | Y | FK → Election |
| position_id | str (UUID7) | Y | FK → ElectivePosition |
| polling_station_id | str (UUID7) | Y | FK → PollingStation |
| session_id | str (UUID7) | Y | FK → PollingSession |
| scanned_image | bytes | Y | Encrypted PDF |
| scanned_image_hash | str(64) | Y | SHA-256; authoritative reference |
| qr_code | str(200) | Y | Binds station + kit + form + election |
| digital_signature | str(1024) | Y | Presiding Officer HSM-signed |
| icr_extraction | jsonb | N | AI-extracted vote counts |
| icr_confidence | decimal(3,2) | N | 0.00–1.00 |
| human_reviewed | bool | Y | Required if confidence < threshold |
| human_confirmed_by | str (UUID7) | N | FK → User |
| agent_countersignatures | jsonb | N | Array of party-agent signatures |
| votes_cast_reported | int | N | |
| valid_votes | int | N | |
| rejected_votes | int | N | |
| status | enum | Y | `SCANNED`, `TRANSMITTED`, `VERIFIED`, `PUBLISHED`, `DISPUTED` |
| merkle_inclusion_proof | jsonb | N | Path in Merkle log |

**Owner:** M10. **Lifecycle:** immutable once signed; ICR may be updated as new versioned row. **Retention:** permanent (Constitution Art. 138). **PII:** NONE (ballot secrecy).
**Events emitted:** `form34a.submitted`, `form34a.verified`, `form34a.published`.

---

#### Form34B
| Field | Type | Req | Notes |
|---|---|---|---|
| form_id | str (UUID7) | Y | |
| election_id | str (UUID7) | Y | |
| position_id | str (UUID7) | Y | |
| constituency_id | str (UUID7) | Y | |
| returning_officer_id | str (UUID7) | Y | |
| stations_reported | int | Y | |
| stations_expected | int | Y | |
| aggregated_counts | jsonb | Y | Per-candidate totals |
| ro_signature | str(1024) | Y | |
| derived_from_forms | list[str] | Y | Form34A IDs |
| merkle_root | str(64) | Y | Root of contributing forms |
| status | enum | Y | `DRAFT`, `SIGNED`, `PUBLISHED`, `DISPUTED` |

**Owner:** M10. **Lifecycle:** immutable once signed. **Retention:** permanent. **PII:** NONE.

---

#### Form34C
Same structure as Form34B, scoped to county or national level. **Owner:** M10.

---

#### Forms35-38 series
Structurally identical to Form34A/B/C, one series per elective position:
- **Form35** — Governor / Deputy Governor
- **Form36** — Senator
- **Form37** — MP
- **Form38** — CWR / MCA

Each has A (station), B (constituency/county), C (national/county summary) variants. **Owner:** M10 for all.

---

#### ResultsPublicationSnapshot
| Field | Type | Req | Notes |
|---|---|---|---|
| snapshot_id | str (UUID7) | Y | |
| election_id | str (UUID7) | Y | |
| published_at | datetime | Y | |
| snapshot_type | enum | Y | `PROVISIONAL`, `UPDATE`, `FINAL` |
| covered_forms | jsonb | Y | Manifest of included Form 34A/B/C |
| merkle_root | str(64) | Y | Root of everything in snapshot |
| signature | str(1024) | Y | IEBC HSM signature |
| cdn_asset_url | str(500) | Y | Signed CDN URL |
| transparency_log_index | int | Y | Position in national transparency log |

**Owner:** M11. **Lifecycle:** immutable. **Retention:** permanent. **PII:** NONE.

---

#### ReproducibilityBundle
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| election_phase | str | Y | e.g. `general_2027` |
| bundle_version | str | Y | Semantic version |
| manifest_hash_sha256 | str(64) | Y | |
| form_count | int | Y | |
| bundle_size_bytes | int | Y | |
| published_at | datetime | Y | |
| publication_channels | list[str] | Y | URLs (IEBC site, GitHub, IPFS, Gazette, radio bulletin) |
| expected_stage_hashes | dict | Y | stage_name → sha256 |

**Owner:** M14. **Lifecycle:** immutable. **Retention:** permanent. **PII:** NONE.

---

#### ParticipationToken
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| token_sha256 | str(64) | Y | SHA-256 of 256-bit random token; token itself discarded |
| station_code | str | Y | |
| issued_at | datetime | Y | |
| kit_serial | str | Y | |
| included_in_bundle | bool | Y | Whether in public reproducibility bundle |

**Owner:** M14. **Lifecycle:** immutable. **Retention:** 15 years. **PII:** NONE (token hash only; no voter link possible).
**Note:** Ballot secrecy design — the token proves presence without linking to identity.

---

#### AuditRun
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| election_id | str | Y | FK → Election |
| contest_id | str | Y | FK → ElectivePosition |
| risk_limit_alpha | float | Y | Default 0.05 |
| margin_of_victory | float | Y | Derived from Form34C |
| seed_nist | str | Y | NIST Randomness Beacon |
| seed_bitcoin_block_hash | str | Y | |
| seed_composite | str | Y | XOR of 5 independent sources |
| seed_drawn_at | datetime | Y | |
| sample_size_target | int | Y | |
| manifest_hash | str(64) | Y | SHA-256 of published ballot manifest |
| status | enum | Y | `PENDING`, `DRAWING`, `ACTIVE`, `CONFIRMED`, `RECOUNT_REQUIRED` |
| outcome_signed_at | datetime | N | |

**Owner:** M21. **Lifecycle:** immutable once sealed. **Retention:** permanent. **PII:** NONE.

---

#### SerialRangeAssignment
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| station_code | str | Y | |
| serial_low | int | Y | |
| serial_high | int | Y | |
| book_count | int | Y | |
| assigned_at | datetime | Y | |
| assigning_officer_id | str | Y | FK → User |
| seal_uids | list[str] | Y | Tamper seal UIDs on bundle |
| m39_manifest_ref | str | Y | FK → PrintOrder |

**Owner:** M63. **Lifecycle:** immutable. **Retention:** permanent. **PII:** NONE.

---

#### SpoiltBallotEvent
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| station_code | str | Y | |
| spoilt_serial | int | Y | |
| reason | enum | Y | `INK_BLEED`, `MIS_MARK`, `DOUBLE_MARK`, `VOTER_REQUEST` |
| replacement_serial | int | N | |
| recorded_at | datetime | Y | |
| presiding_officer_id | str | Y | FK → User |

**Owner:** M63 (physical) and M27 (intent preservation context). **Lifecycle:** immutable. **Retention:** permanent. **PII:** NONE.

---

#### PrintOrder
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| election_id | str | Y | FK → Election |
| contest_id | str | Y | FK → ElectivePosition |
| constituency_code | str | Y | |
| quantity_authorised | int | Y | |
| serial_range_start | int | Y | |
| serial_range_end | int | Y | |
| paper_spec_hash | str(64) | Y | SHA-256 of approved paper specification |
| issued_by_officer_id | str | Y | FK → User |
| countersigned_by_commissioner_id | str | Y | FK → User |
| issued_at | datetime | Y | |
| gazette_notice_id | str | N | FK → GazetteNotice |

**Owner:** M39. **Lifecycle:** immutable once countersigned. **Retention:** permanent. **PII:** NONE.

---

### 3.6 KitInventory Context

#### Kit
| Field | Type | Req | Notes |
|---|---|---|---|
| kit_id | str (UUID7) | Y | |
| serial_number | str(30) | Y | OEM-issued |
| model | str(50) | Y | |
| firmware_version | str(30) | Y | |
| firmware_hash | str(64) | Y | Expected image hash |
| certificate_id | str (UUID7) | Y | FK → KitCertificate |
| status | enum | Y | `RECEIVED`, `IMAGING`, `PROVISIONED`, `DEPLOYED`, `ACTIVE`, `WARRANTY_REPAIR`, `LOST`, `RETIRED`, `DESTROYED` |
| current_polling_station_id | str | N | |
| current_custodian_id | str | N | FK → User |
| warranty_expires | date | Y | |
| last_boot_attestation | jsonb | N | Bootloader + kernel + app hashes |
| last_boot_attestation_at | datetime | N | |
| tamper_seal_ids | list[str] | Y | Applied seal IDs |

**Owner:** M20. **Lifecycle:** versioned-mutable. **Retention:** permanent. **PII:** NONE.
**Index:** `serial_number` (unique), `status`, `current_polling_station_id`.

---

#### KitCertificate
| Field | Type | Req | Notes |
|---|---|---|---|
| certificate_id | str (UUID7) | Y | |
| kit_id | str (UUID7) | Y | FK → Kit |
| issued_at | datetime | Y | |
| expires_at | datetime | Y | |
| revoked_at | datetime | N | |
| revocation_reason | enum | N | `LOST`, `STOLEN`, `COMPROMISED`, `DECOMMISSIONED` |
| cert_serial | str(64) | Y | X.509 serial |
| cert_pem | str | Y | Full X.509 certificate |
| tpm_binding | jsonb | Y | TPM attestation on issue |

**Owner:** P04 (PKI). **Lifecycle:** immutable; revocation is separate record. **Retention:** permanent. **PII:** NONE.

---

#### TamperSeal
| Field | Type | Req | Notes |
|---|---|---|---|
| seal_id | str (UUID7) | Y | |
| serial_number | str(30) | Y | Cryptographic serial in QR |
| status | enum | Y | `UNAPPLIED`, `APPLIED`, `INSPECTED_INTACT`, `TAMPERED` |
| applied_at | datetime | N | |
| applied_by | str (UUID7) | N | FK → User |
| applied_gps | geography | N | |
| applied_signature | str(1024) | N | Biometric-signed |
| target_entity | enum | Y | `KIT`, `TRANSPORT_CASE`, `RESULT_ENVELOPE` |
| target_id | str (UUID7) | Y | FK |

**Owner:** M20. **Lifecycle:** immutable once applied. **Retention:** permanent. **PII:** NONE.

---

#### KitCustodyEvent
| Field | Type | Req | Notes |
|---|---|---|---|
| event_id | str (UUID7) | Y | |
| kit_id | str (UUID7) | Y | FK → Kit |
| from_custodian_id | str (UUID7) | Y | FK → User |
| to_custodian_id | str (UUID7) | Y | FK → User |
| event_type | enum | Y | `RECEIVED`, `IMAGED`, `DISPATCHED`, `ASSIGNED_TO_STATION`, `RETURNED`, `WARRANTY_SHIPPED`, `RETIRED` |
| timestamp | datetime | Y | |
| seal_verified | bool | Y | |
| signed_event | bytes | Y | Both parties biometric-sign |

**Owner:** M20. **Lifecycle:** immutable, event-sourced. **Retention:** permanent. **PII:** NONE.
**Kafka topic:** `kit.custody`.

---

#### Asset
| Field | Type | Req | Notes |
|---|---|---|---|
| uid | str (UUID7) | Y | KE27-format SKU-scoped |
| sku | str | Y | |
| parent_uid | str | N | FK → Asset (hierarchy) |
| state | str | Y | `IN_STOCK`, `DEPLOYED`, `DECOMMISSIONED` |
| condition | enum | Y | `NEW`, `GOOD`, `FAULTY`, `DECOMMISSIONED` |
| factory_serial | str | Y | |
| batch_id | str | Y | |
| current_facility_id | str | N | FK → Facility |

**Owner:** M20. **Lifecycle:** versioned-mutable. **Retention:** permanent. **PII:** NONE.

---

#### Facility
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| facility_type | enum | Y | `FACTORY`, `PORT`, `WAREHOUSE`, `RDC`, `CONSTITUENCY`, `STATION` |
| name | str | Y | |
| gps_lat | float | Y | |
| gps_lng | float | Y | |
| county_code | str | N | |
| constituency_code | str | N | |
| jurisdiction_level | enum | Y | `NATIONAL`, `REGIONAL`, `COUNTY`, `CONSTITUENCY`, `STATION` |

**Owner:** M20. **Lifecycle:** mutable. **Retention:** permanent. **PII:** NONE.

---

### 3.7 PollingOperations Context

#### QueueSnapshot
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| station_code | str | Y | |
| snapshot_at | datetime | Y | |
| input1_po_estimate | int | N | Presiding officer manual estimate |
| input2_throughput_per_15min | int | N | From authenticated voter count |
| derived_queue_length | int | N | |
| derived_wait_min | int | N | |
| alert_level | enum | N | `GREEN`, `AMBER`, `RED`, `CRITICAL` |
| kit_serial | str | Y | |
| connectivity | bool | Y | Was kit connected at snapshot time |

**Owner:** M19. **Lifecycle:** append-only. **Retention:** 15 years. **PII:** NONE.
**Kafka topic:** `queue.snapshot`.

---

#### SpecialVoterFacility
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| facility_type | enum | Y | `HOSPITAL`, `PRISON`, `REMAND_HOME`, `IDP_CAMP`, `DIPLOMATIC_MISSION` |
| name | str | Y | |
| constituency_code | str | Y | |
| officer_id | str | Y | FK → User (assigned RO) |
| election_id | str | Y | FK → Election |
| capacity | int | Y | |

**Owner:** M25. **Lifecycle:** mutable. **Retention:** 15 years. **PII:** NONE.

---

#### EmbassyStation
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| country_iso | str(3) | Y | ISO 3166-1 alpha-3 |
| city | str | Y | |
| consulate_name | str | Y | |
| coordinator_officer_id | str | Y | FK → User |
| election_id | str | Y | FK → Election |
| kit_count | int | Y | |
| registered_diaspora_voters | int | Y | |

**Owner:** M24. **Lifecycle:** per-election. **Retention:** 15 years. **PII:** NONE.

---

#### DiasporaVoterRoll
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| voter_id | str | Y | FK → Voter (canonical) |
| embassy_station_id | str | Y | FK → EmbassyStation |
| registration_status | enum | Y | `PENDING`, `APPROVED`, `REJECTED` |
| approved_at | datetime | N | |

**Owner:** M24. **Lifecycle:** per-election. **Retention:** contract + 5 years. **PII:** MED.

---

#### PreMarkValidationEvent
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| station_code | str | Y | |
| contest_id | str | Y | FK → ElectivePosition |
| opted_in | bool | Y | |
| result_code | enum | N | `VALID`, `UNMARKED`, `MULTIPLE_MARKS`, `AMBIGUOUS_MARK`, `WRONG_BALLOT` |
| voter_proceeded | bool | N | |
| event_at | datetime | Y | |

**Owner:** M27. **Lifecycle:** immutable, event-sourced. **Retention:** permanent. **PII:** NONE.
**Design note:** no voter_id field — station-aggregate only to preserve ballot secrecy.

---

### 3.8 AuditCompliance Context

#### AuditEvent
| Field | Type | Req | Notes |
|---|---|---|---|
| event_id | str (UUID7) | Y | |
| actor_id | str (UUID7) | Y | User, Kit, or Service |
| actor_type | enum | Y | `USER`, `KIT`, `SERVICE`, `SYSTEM` |
| action | str(100) | Y | e.g. `voter.register`, `form34a.submit` |
| target_entity_type | str(50) | Y | |
| target_entity_id | str (UUID7) | Y | |
| ip_address | inet | N | |
| gnss_fix | geography | N | For device-originated events |
| timestamp | datetime | Y | RFC 3161 TSA timestamped |
| origin_signature | str(1024) | Y | Actor signature |
| backend_countersignature | str(1024) | Y | HSM signature |
| constituency_id | str (UUID7) | Y | Sharding key for Merkle log |
| merkle_log_index | int | Y | Position in per-constituency log |
| merkle_leaf_hash | str(64) | Y | |
| event_payload | jsonb | Y | Structured event body |

**Owner:** P05 (Audit). **Lifecycle:** immutable, append-only. **Retention:** 90 days online (ClickHouse); permanent archive (object store + escrow). **PII:** MED.
**Kafka topic:** `audit.event`.

---

#### MerkleLogRoot
| Field | Type | Req | Notes |
|---|---|---|---|
| root_id | str (UUID7) | Y | |
| constituency_id | str (UUID7) | Y | |
| root_hash | str(64) | Y | |
| leaf_count | int | Y | |
| published_at | datetime | Y | |
| transparency_log_index | int | Y | National log position |
| publication_signature | str(1024) | Y | HSM-signed |

**Owner:** P05. **Lifecycle:** immutable. **Retention:** permanent. **PII:** NONE.

---

#### DPIA
| Field | Type | Req | Notes |
|---|---|---|---|
| dpia_id | str (UUID7) | Y | |
| subsystem | str(50) | Y | e.g. BVR, VPRS |
| version | int | Y | |
| assessed_at | datetime | Y | |
| assessed_by | str (UUID7) | Y | FK → User |
| risks_identified | jsonb | Y | Ranked list |
| mitigations | jsonb | Y | |
| approved_by | str (UUID7) | Y | ODPC officer |

**Owner:** P06 (Compliance). **Lifecycle:** versioned. **Retention:** permanent. **PII:** NONE.

---

#### DataBreachNotification
| Field | Type | Req | Notes |
|---|---|---|---|
| notification_id | str (UUID7) | Y | |
| detected_at | datetime | Y | |
| notified_odpc_at | datetime | N | Must be ≤ 72 h after detection |
| description | str | Y | |
| affected_records | int | Y | |
| mitigation_taken | str | Y | |
| status | enum | Y | `DETECTED`, `NOTIFIED`, `INVESTIGATED`, `RESOLVED` |

**Owner:** P06. **Lifecycle:** mutable. **Retention:** permanent. **PII:** NONE.

---

#### SubjectAccessRequest
| Field | Type | Req | Notes |
|---|---|---|---|
| sar_id | str (UUID7) | Y | |
| requester_id_number | str(20) | Y | Encrypted |
| request_type | enum | Y | `ACCESS`, `RECTIFICATION`, `ERASURE`, `PORTABILITY`, `RESTRICTION` |
| received_at | datetime | Y | |
| due_at | datetime | Y | received_at + 30 days |
| status | enum | Y | `RECEIVED`, `VERIFIED`, `PROCESSING`, `RESPONDED`, `REJECTED` |

**Owner:** P06. **Lifecycle:** mutable. **Retention:** 7 years. **PII:** HIGH.

---

#### IABEscalation
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| level | int | Y | 1–4 |
| triggered_by | str | Y | FK → User (IAB member) |
| triggered_at | datetime | Y | |
| trigger_description | str | Y | |
| linked_event_ids | list[str] | Y | AuditEvent UUIDs |
| iebc_response_due | datetime | Y | |
| public | bool | Y | Whether published to public |
| signed_hash | str(64) | Y | HSM signature |

**Owner:** M15. **Lifecycle:** immutable once filed. **Retention:** permanent. **PII:** NONE.

---

#### TransmissionEvent
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| kit_serial | str | Y | |
| form_id | str | Y | FK → Form34A |
| channel | enum | Y | `LTE`, `4G`, `3G`, `SMS`, `LORA`, `SATELLITE` |
| transmitted_at | datetime | Y | |
| received_at | datetime | N | |
| latency_ms | int | N | |
| retry_count | int | Y | |
| encryption_algo | str | Y | |
| lawful_basis_ref | str | Y | Elections Act §39 reference |
| verdict | enum | N | `COMPLIANT`, `NONCOMPLIANT`, `FLAGGED` |

**Owner:** M62. **Lifecycle:** immutable. **Retention:** 15 years. **PII:** NONE.

---

#### EvidenceBundle
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| petition_or_case_id | str | Y | |
| requestor_type | enum | Y | `COURT`, `PETITIONER`, `RESPONDENT`, `IAB`, `AG`, `AUDITOR_GENERAL` |
| requestor_ref | str | Y | |
| created_at | datetime | Y | |
| items | list[BundleItem] | Y | Up to 15 items per §XII.4 |
| bundle_hash | str(64) | Y | SHA-256 of manifest |
| served_at | datetime | N | |
| access_log | list[str] | Y | Log of who accessed what |

**Owner:** M40. **Lifecycle:** immutable once sealed. **Retention:** 15 years. **PII:** MED.

---

#### RedTeamEngagement
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| election_cycle | str | Y | |
| scope | str | Y | Modules in scope |
| contractor | str | Y | External red team firm |
| start_date | date | Y | |
| end_date | date | N | |
| findings_count | int | N | |
| status | enum | Y | `PLANNED`, `ACTIVE`, `COMPLETED`, `REMEDIATED` |

**Owner:** M61. **Retention:** 15 years. **PII:** NONE.

---

### 3.9 FieldOps Context

#### GrievanceCase
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| intake_channel | enum | Y | `VERIFIER_APP`, `SHORT_CODE`, `WHATSAPP`, `SMS`, `KIT_KIOSK`, `IN_PERSON` |
| category | enum | Y | `VIOLENCE`, `KIT_FAILURE`, `BALLOT_TAMPERING`, `IMPERSONATION`, `DOUBLE_VOTING`, `BRIBERY`, `VOTER_EXCLUSION`, `AGENT_EXPELLED`, `FORM_NOT_POSTED`, `QUEUE`, `VOTER_ROLL_QUERY`, `PROCESS`, `FEEDBACK` |
| severity | enum | Y | `P0`, `P1`, `P2` |
| station_code | str | N | |
| description_text | str | Y | |
| contact_hash | str(64) | N | SHA-256 of MSISDN/email |
| is_whistleblower | bool | Y | |
| status | enum | Y | `OPEN`, `TRIAGED`, `ASSIGNED`, `RESOLVED`, `ESCALATED`, `CLOSED` |
| created_at | datetime | Y | |

**Owner:** M23. **Lifecycle:** mutable. **Retention:** 15 years. **PII:** MED.

---

#### ECCAlert
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| source_module | str | Y | e.g. `M26`, `M19`, `M06` |
| severity | enum | Y | `P0`, `P1`, `P2`, `P3` |
| category | enum | Y | `KIT_OFFLINE`, `THRESHOLD_BREACH`, `S44A`, `SAFETY`, `COMMS`, `ANOMALY` |
| station_code | str | N | |
| constituency_code | str | N | |
| description | str | Y | |
| acknowledged_by | str | N | FK → User |
| resolved_at | datetime | N | |

**Owner:** M16. **Lifecycle:** mutable until resolved. **Retention:** 15 years. **PII:** NONE.
**Kafka topic:** `ecc.alert`.

---

#### DetectionAlert
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| layer | enum | Y | `L1` (turnout) through `L7` (cross-source) |
| severity | enum | Y | `INFO`, `WARNING`, `CRITICAL` |
| station_code | str | N | |
| description | str | Y | |
| contributing_signals | list[str] | Y | Source event IDs |
| anomaly_score | float | N | |
| generated_at | datetime | Y | |
| resolution | enum | N | `DISMISSED`, `CONFIRMED`, `ESCALATED` |

**Owner:** M26. **Lifecycle:** mutable. **Retention:** 15 years. **PII:** NONE.
**Kafka topic:** `anomaly.alert`.

---

#### ContentFlag (speech integrity)
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| platform | str | Y | |
| content_url | str | Y | |
| content_hash_sha256 | str(64) | Y | |
| captured_at | datetime | Y | |
| language | str | Y | BCP-47 |
| detection_type | enum | Y | `HATE_SPEECH`, `CIB`, `DEEPFAKE`, `SUPPRESSION`, `DISRUPTION` |
| confidence | float | Y | |
| severity | enum | Y | `HIGH`, `MEDIUM`, `LOW` |
| analyst_decision | enum | N | `CONFIRM`, `DISMISS`, `ESCALATE` |

**Owner:** M17. **Lifecycle:** mutable. **Retention:** 15 years. **PII:** NONE.

---

#### KitTelemetrySnapshot
| Field | Type | Req | Notes |
|---|---|---|---|
| kit_serial | str | Y | |
| station_code | str | Y | |
| snapshot_at | datetime | Y | |
| online | bool | Y | |
| battery_pct | int | Y | |
| gps_lat | float | Y | |
| gps_lng | float | Y | |
| geofence_ok | bool | Y | |
| firmware_version | str | Y | |
| auth_throughput_per_hour | int | Y | |
| biometric_first_try_success_pct | float | Y | |
| tx_queue_depth | int | Y | |

**Owner:** M16. **Lifecycle:** append-only time-series. **Retention:** 90 days online; 15 years archive. **PII:** NONE.
**Kafka topic:** `kit.telemetry`.

---

#### AgentCountersignature
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| form34a_id | str (UUID7) | Y | FK → Form34A |
| agent_user_id | str (UUID7) | Y | FK → User (party agent) |
| party_code | str(10) | Y | |
| station_code | str | Y | |
| signed_at | datetime | Y | |
| agent_biometric_hash | str(64) | Y | On-device biometric verification hash |
| countersignature_bytes | bytes | Y | Ed25519 signature over form34a_id + timestamp |
| device_attestation | jsonb | Y | Mobile app TPM/Play Integrity attestation |

**Owner:** M30 (Mobile Party Agent) / M10 (RTS Backend countersign endpoint). **Lifecycle:** immutable. **Retention:** permanent. **PII:** LOW (party affiliation only).
**Kafka topic:** `agent.countersign`.

---

#### ObserverVisitLog
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| observer_id | str (UUID7) | Y | FK → IndividualObserver |
| station_code | str | Y | |
| visit_type | enum | Y | `OPENING`, `DURING_POLLING`, `CLOSE`, `COUNTING`, `RESULTS_POSTING` |
| arrived_at | datetime | Y | GPS-timestamped |
| departed_at | datetime | N | |
| gps_lat | float | Y | |
| gps_lng | float | Y | |
| findings_summary | str | N | Optional narrative |
| incident_ids | list[str] | N | FK list → GrievanceCase |
| signature_hash | str(64) | Y | Ed25519 over id + arrived_at |

**Owner:** M34 (Mobile Observer). **Lifecycle:** immutable, append-only. **Retention:** 15 years. **PII:** LOW.
**Kafka topic:** `observer.visit`.

---

#### PersonnelAttendanceRecord
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| staff_id | str (UUID7) | Y | FK → User (temporary polling staff) |
| station_code | str | Y | |
| role | enum | Y | `PRESIDING_OFFICER`, `DEPUTY_PO`, `POLL_CLERK`, `SECURITY`, `LOGISTICS` |
| checked_in_at | datetime | Y | GPS-verified |
| checked_out_at | datetime | N | |
| gps_lat | float | Y | |
| gps_lng | float | Y | |
| geofence_ok | bool | Y | Within 100m of assigned station |
| payroll_trigger_sent | bool | Y | |

**Owner:** M33 (Mobile Personnel Dispatch). **Lifecycle:** immutable, event-sourced. **Retention:** 7 years (PPADA 2015). **PII:** MED.
**Kafka topic:** `staff.attendance`.

---

#### ContinuousVoterRegisterEvent
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| voter_id | str (UUID7) | Y | FK → Voter |
| event_type | enum | Y | `DEATH_PROPOSED`, `DEATH_CONFIRMED`, `ID_CORRECTION`, `DORMANCY_FLAG`, `FAMILY_NOTIFICATION`, `REMOVAL_APPROVED`, `REMOVAL_REJECTED`, `REINSTATEMENT`, `WARD_TRANSFER` |
| source | enum | Y | `CRS`, `NRB`, `OFFICER_TRIAGE`, `APPEAL_OUTCOME`, `BATCH_AUDIT` |
| source_reference | str | N | External notification ID |
| proposed_by_user_id | str (UUID7) | N | FK → User (officer) |
| dual_control_user_id | str (UUID7) | N | Second-officer countersignature |
| event_at | datetime | Y | |
| evidence_url | str | N | Supporting document reference |
| approved_at | datetime | N | |

**Owner:** M35 (CVR). **Lifecycle:** immutable, event-sourced. **Retention:** permanent. **PII:** MED.
**Kafka topic:** `cvr.lifecycle`.

---

#### BallotPrintOrder
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| election_id | str (UUID7) | Y | FK → Election |
| contest_id | str (UUID7) | Y | FK → ElectivePosition |
| printer_partner | enum | Y | `AL_GHURAIR`, `GOVT_PRINTER` |
| quantity_ordered | int | Y | |
| quantity_printed | int | N | Confirmed at QC scan |
| quantity_spoilt | int | N | |
| serial_range_start | str(20) | Y | First ballot serial in run |
| serial_range_end | str(20) | Y | Last ballot serial in run |
| print_run_id | str(30) | Y | Printer-assigned run ID |
| status | enum | Y | `ORDERED`, `PRINTING`, `QC_PASSED`, `QC_FAILED`, `SHIPPED`, `RECEIVED`, `DISTRIBUTED` |
| countersigned_by | str (UUID7) | N | IEBC officer FK → User |
| rfid_manifest_id | str (UUID7) | N | FK → PrintLotManifest |

**Owner:** M39 (Ballot Printing Chain). **Lifecycle:** versioned-mutable. **Retention:** permanent. **PII:** NONE.

---

#### PrintLotManifest
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| print_order_id | str (UUID7) | Y | FK → BallotPrintOrder |
| lot_number | int | Y | |
| bundle_count | int | Y | Bundles of 50 |
| rfid_tag_ids | list[str] | Y | List of RFID tag serials |
| warehouse_arrival_at | datetime | N | |
| warehouse_officer_id | str | N | FK → User |
| arrival_signature | bytes | N | Ed25519 signature |

**Owner:** M39. **Lifecycle:** immutable once signed. **Retention:** permanent. **PII:** NONE.

---

### 3.10 AccreditationObservers Context

#### MediaAccreditation
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| organisation_id | str | Y | FK → MediaOrganisation |
| journalist_name | str | Y | |
| id_document_ref | str(64) | Y | Hash of ID scan |
| access_zones | list[AccessZone] | Y | |
| status | enum | Y | `SUBMITTED`, `UNDER_VETTING`, `APPROVED`, `REJECTED`, `BADGE_ISSUED`, `SUSPENDED`, `REVOKED` |
| badge_number | str | N | |
| valid_from | date | N | |
| valid_to | date | N | |

**Owner:** M42. **Lifecycle:** mutable. **Retention:** 15 years. **PII:** MED.

---

#### ObserverOrganisation
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| org_type | enum | Y | `DOMESTIC_NGO`, `INTERNATIONAL_MISSION`, `DIPLOMATIC_MISSION`, `RELIGIOUS_BODY`, `ACADEMIC` |
| name | str | Y | |
| country_iso | str(3) | N | |
| accreditation_number | str | N | |
| lead_coordinator_id | str | N | FK → IndividualObserver |
| status | enum | Y | `SUBMITTED`, `APPROVED`, `REJECTED`, `ACTIVE`, `SUSPENDED` |

**Owner:** M43. **Lifecycle:** mutable. **Retention:** 15 years. **PII:** MED.

---

#### ObserverMission
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| organisation_id | str | Y | FK → ObserverOrganisation |
| election_id | str | Y | FK → Election |
| head_of_mission | str | Y | |
| observer_count | int | Y | |
| mou_signed_at | datetime | N | |
| deployment_plan_id | str | N | FK → DeploymentPlan |
| status | enum | Y | `PROVISIONAL`, `CONFIRMED`, `DEPLOYED`, `COMPLETED` |

**Owner:** M57. **Lifecycle:** mutable. **Retention:** 15 years. **PII:** MED.

---

### 3.11 Finance Context

#### CandidateDisclosure
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| candidate_id | str | Y | FK → Candidate |
| period_start | date | Y | |
| period_end | date | Y | |
| declared_total_kes | int | Y | Integer cents |
| ceiling_kes | int | Y | ECFA 2013 s.22 ceiling |
| ceiling_pct_used | float | Y | |
| status | enum | Y | `DRAFT`, `SUBMITTED`, `UNDER_REVIEW`, `ACCEPTED`, `QUERIED` |
| discrepancy_flag | bool | Y | |
| reviewed_by | str | N | FK → User |

**Owner:** M22. **Lifecycle:** mutable. **Retention:** 15 years. **PII:** MED.

---

#### ElectionBudget
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| election_id | str | Y | FK → Election |
| total_approved_kes | int | Y | Integer cents |
| nt_vote_ref | str | Y | National Treasury vote reference |
| approved_by | str | Y | FK → User (PS Treasury signatory) |
| status | enum | Y | `DRAFT`, `SUBMITTED`, `APPROVED`, `REVISED`, `AUDITED` |

**Owner:** M54. **Lifecycle:** versioned. **Retention:** 15 years. **PII:** NONE.

---

#### PayrollRun
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| pay_period_start | date | Y | |
| pay_period_end | date | Y | |
| election_id | str | Y | FK → Election |
| staff_count | int | Y | |
| gross_total_kes | int | Y | Integer cents |
| net_total_kes | int | Y | Integer cents |
| status | enum | Y | `DRAFT`, `APPROVED`, `DISBURSED`, `AUDITED` |
| approved_by | str | Y | FK → User |

**Owner:** M55. **Lifecycle:** mutable until disbursed. **Retention:** 15 years. **PII:** NONE (individual payroll lines carry PII).

---

### 3.12 IdentityAccess Context

#### User
| Field | Type | Req | Notes |
|---|---|---|---|
| user_id | str (UUID7) | Y | |
| username | str(100) | Y | |
| email | str(320) | Y | Encrypted |
| mobile | str(15) | Y | Encrypted |
| status | enum | Y | `ACTIVE`, `SUSPENDED`, `DEACTIVATED` |
| password_hash | str(200) | Y | Argon2id |
| mfa_type | enum | Y | `FIDO2`, `SMART_CARD`, `BIOMETRIC` |
| id_number | str(20) | Y | Kenya national ID (encrypted) |

**Owner:** P04 (IAM). **Lifecycle:** versioned. **Retention:** employment + 7 years. **PII:** HIGH.

---

#### Role
| Field | Type | Req | Notes |
|---|---|---|---|
| role_id | str (UUID7) | Y | |
| name | enum | Y | `ENROLMENT_CLERK`, `PRESIDING_OFFICER`, `RETURNING_OFFICER`, `COUNTY_RETURNING_OFFICER`, `PARTY_AGENT`, `IEBC_ADMIN`, `IEBC_COMMISSIONER`, `AUDITOR`, `SUPPORT_L1`, `SUPPORT_L2`, `ODPC_OFFICER`, `OBSERVER`, `MEDIA`, `COURT_REGISTRAR`, `AG_OFFICE`, `HEALTH_OFFICER` |
| description | str | Y | |

**Owner:** P04. **Lifecycle:** immutable. **Retention:** permanent. **PII:** NONE.

---

#### UserRoleAssignment
| Field | Type | Req | Notes |
|---|---|---|---|
| assignment_id | str (UUID7) | Y | |
| user_id | str (UUID7) | Y | FK → User |
| role_id | str (UUID7) | Y | FK → Role |
| scope_type | enum | Y | `GLOBAL`, `COUNTY`, `CONSTITUENCY`, `WARD`, `POLLING_STATION` |
| scope_id | str (UUID7) | Y | FK to area entity |
| effective_from | datetime | Y | |
| effective_to | datetime | N | |
| granted_by | str (UUID7) | Y | FK → User |
| grant_signature | str(1024) | Y | HSM-signed |

**Owner:** P04. **Lifecycle:** immutable (revocation is a separate end-dated record). **Retention:** permanent. **PII:** NONE.

---

#### PrivilegedSession
| Field | Type | Req | Notes |
|---|---|---|---|
| session_id | str (UUID7) | Y | |
| user_id | str (UUID7) | Y | FK → User |
| bastion_id | str(50) | Y | |
| session_start | datetime | Y | |
| session_end | datetime | N | |
| recording_url | str(500) | Y | Encrypted session recording |
| justification_ticket | str(50) | Y | External ticket ID |
| approved_by | str (UUID7) | Y | FK → User |

**Owner:** P04. **Lifecycle:** immutable. **Retention:** 5 years. **PII:** MED.

---

### 3.13 Operational Context

#### SupportTicket
| Field | Type | Req | Notes |
|---|---|---|---|
| ticket_id | str (UUID7) | Y | |
| severity | enum | Y | `SEV_1`, `SEV_2`, `SEV_3` |
| channel | enum | Y | `PHONE`, `WHATSAPP`, `EMAIL`, `IN_APP`, `WAR_ROOM` |
| polling_station_id | str | N | FK → PollingStation |
| kit_id | str | N | FK → Kit |
| opened_at | datetime | Y | |
| resolved_at | datetime | N | |
| sla_response_met | bool | N | 2h/4h/6h per severity |

**Owner:** O02 (Support). **Lifecycle:** mutable. **Retention:** 5 years. **PII:** MED.

---

#### AIModel
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| code | str | Y | e.g. `A1`, `B1`, `C1` |
| name | str | Y | |
| version | str | Y | Semantic version |
| layer | enum | Y | `KIT`, `CENTRAL`, `CITIZEN`, `DISINFORMATION`, `INTERNAL` |
| eval_accuracy | float | N | |
| eval_bias_flag | bool | Y | |
| known_limitations | str | Y | |
| human_in_loop | str | Y | Description of human oversight mechanism |
| model_card_url | str | N | |

**Owner:** M28. **Lifecycle:** versioned. **Retention:** permanent. **PII:** NONE.

---

#### LessonsLearningFinding
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| election_cycle | str | Y | e.g. `2027-KE-GE` |
| submitter_category | enum | Y | `IEBC_DEPT`, `IAB`, `KEOG`, `ELOG`, `EU_EOM`, `DOMESTIC_OBSERVER`, `MEDIA`, `CITIZEN` |
| finding_type | enum | Y | `TECHNICAL`, `PROCESS`, `LEGAL`, `SECURITY`, `ACCESSIBILITY`, `COMMUNICATION` |
| severity | enum | Y | `HIGH`, `MEDIUM`, `LOW` |
| description | str | Y | |
| recommendation_ids | list[str] | Y | FK list → Recommendation |
| status | enum | Y | `DRAFT`, `PUBLISHED`, `REFERRED`, `CLOSED` |

**Owner:** M41. **Lifecycle:** mutable. **Retention:** permanent. **PII:** NONE.

---

#### DataSource (archive registry)
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| system_id | str | Y | Source module ID e.g. `M14`, `M10` |
| category | enum | Y | `RESULTS`, `VOTER_ROLL`, `CHAIN_OF_CUSTODY`, `FINANCIAL`, `AUDIT_LOG`, `REPRODUCIBILITY`, `FORENSICS` |
| retention_years | int | Y | 7, 10, or 15 |
| classification | enum | Y | `PUBLIC`, `RESTRICTED`, `RESTRICTED_JUDICIAL` |
| pii_flag | bool | Y | |
| lawful_basis | str | Y | DPA 2019 §26 reference |

**Owner:** M69. **Lifecycle:** immutable. **Retention:** per retention_years field. **PII:** NONE.

---

#### ArchiveCollection
| Field | Type | Req | Notes |
|---|---|---|---|
| id | str (UUID7) | Y | |
| source_id | str | Y | FK → DataSource |
| election_cycle | str | Y | e.g. `2027_general` |
| ingested_at | datetime | Y | |
| record_count | int | Y | |
| manifest_sha256 | str(64) | Y | |
| format | enum | Y | `CSV`, `JSON_LD`, `PDF_A3`, `BAGIT` |
| storage_locations | list[str] | Y | URIs (primary + KNADS + escrow) |
| knads_accession | str | N | Kenya National Archives accession number |
| doi | str | N | Registered DOI |
| embargo_lifts | date | N | |
| retention_expires | date | Y | |
| status | enum | Y | `ACTIVE`, `MIGRATING`, `EXPIRED`, `DESTROYED` |

**Owner:** M69. **Lifecycle:** mutable. **Retention:** per source retention_years. **PII:** conditional.

---

## 4. Event Catalogue

All events use CloudEvents 1.0 envelope with a mandatory `signature` extension field (HSM countersignature).

| Event | Kafka Topic | Owner | Produced by | Consumed by | Retention |
|---|---|---|---|---|---|
| `voter.enrolled` | `bvr.enrollments` | M02 | BVR backend | ABIS (M03), Audit (P05), BI | 7 days hot; permanent archive |
| `voter.updated` | `bvr.updates` | M02 | BVR backend | BVI (M06), CVR (M35), Audit | 7 days hot |
| `voter.transferred` | `bvr.transfers` | M02 | BVR backend | BVI, CVR, Audit | 7 days hot |
| `voter.register_event` | `cvr.events` | M35 | CVR service | M36, Audit, M64 | permanent |
| `register.snapshot_signed` | `cvr.snapshots` | M35 | Commissioner action | M64, M69, Audit | permanent |
| `deregistration.case_opened` | `deregistration.cases` | M36 | CVR trigger | M36, Audit | permanent |
| `session.opened` | `polling.sessions` | M06 | BVI backend | War-room (M16), Audit | permanent |
| `session.closed` | `polling.sessions` | M06 | BVI backend | RTS (M10), Audit | permanent |
| `voter.authenticated` | `polling.identifications` | M06 | BVI backend | Turnout (M44), Anomaly (M26), Audit | permanent |
| `voter.distress` | `polling.distress` | M06 | BVI backend | War-room (M16), KNPS liaison | permanent |
| `queue.snapshot` | `polling.queue` | M19 | Queue svc | M16, M13, M44 | 90 days |
| `form34a.submitted` | `rts.forms` | M10 | RTS backend | Tally engine, Audit, Portal (M11) | permanent |
| `form34a.verified` | `rts.forms` | M10 | RTS backend | Portal (M11), Transparency log | permanent |
| `form34a.published` | `rts.publication` | M10 | Publication engine | Portal (M11), M13, M14 | permanent |
| `snapshot.published` | `rts.snapshots` | M11 | Publication engine | M13, M14, Transparency log, Escrow | permanent |
| `kit.attestation` | `kit.attestation` | M01/M05/M08 | Kit on boot | PKI (P04), MDM, Audit | permanent |
| `kit.custody` | `kit.custody` | M20 | Inventory | Audit, War-room | permanent |
| `kit.telemetry` | `kit.telemetry` | M01/M05/M08 | Kit heartbeat | M16, M26 | 90 days |
| `security.incident` | `security.incidents` | various | Any module | War-room, SIEM, IAB | permanent |
| `anomaly.alert` | `anomaly.alerts` | M26 | Anomaly engine | M16 (ECC), IAB (M15), Audit | 15 years |
| `ecc.alert` | `ecc.alerts` | M16 | ECC | Audit, IAB | 15 years |
| `grievance.filed` | `grievances` | M23 | Grievance svc | M16, Audit, M13 | 15 years |
| `audit.event` | `audit.events` | P05 | All modules | Merkle builder, Escrow, BI | permanent |
| `boundary.updated` | `boundaries` | M18 | Boundary svc | All map-consuming modules | permanent |
| `election.status_changed` | `elections` | M52 | Election admin | All modules | permanent |
| `candidate.gazetted` | `candidates` | M12 | CRMS | M10, M11, M27, M39, M63 | permanent |
| `nomination_fee.confirmed` | `payments` | M47 | Payment svc | M12, Audit | 15 years |
| `declaration.made` | `declarations` | M51 | Declaration svc | M52, M40, Judiciary, Audit | permanent |
| `bundle.ready` | `evidence_bundles` | M40 | Evidence svc | Court, AG, Audit | 15 years |

---

## 5. Cross-Context Flows

### 5.1 Registration to Polling Day
```
Voter (VoterRegistration) → [voter_id referenced in BVI]
  → VoterIdentificationEvent (PollingOperations)
  → [aggregate count only; no biometric transmitted]
  → TurnoutForecast (FieldOps analytics)
```
ACL: BVI receives a read-model projection of Voter containing only `voter_id`, `polling_station_id`, and `status`. Biometrics stay in the biometric context.

### 5.2 Results Chain
```
Form34A (BallotResults) → Merkle builder → MerkleLogRoot (AuditCompliance)
  → ResultsPublicationSnapshot (BallotResults)
  → ReproducibilityBundle (BallotResults)
  → CDN edge → public verification
```

### 5.3 Candidate to Ballot
```
Candidate (CandidateParty) → [candidate_id, ballot_order, portrait]
  → Form34A.icr_extraction (BallotResults)
  → SerialRangeAssignment (BallotResults)
```

### 5.4 Deregistration Pipeline
```
VoterRegisterEvent (VoterRegistration) → [trigger]
  → DeregistrationCase (VoterRegistration)
  → [if appealed] AppealRecord (VoterRegistration)
  → [outcome] Voter.status = DECEASED/DELETED (VoterRegistration)
  → RegisterSnapshot (VoterRegistration)
```

---

## 6. Reference Data

Static or slowly-changing reference data. Changes require dual-commissioner approval and produce a new version record.

| Reference table | Owner | Records | Update mechanism |
|---|---|---|---|
| County | M18 | 47 | Gazette notice + dual approval |
| Constituency | M18 | 290 | Gazette notice + dual approval |
| Ward | M18 | ~1,450 | Gazette notice + dual approval |
| PollingCentre | M18 | ~35,000 | Pre-election gazette |
| PollingStation | M18 | ~55,393 | Pre-election gazette |
| ElectivePosition types | M52 | 6 standard + referendum | Constitutional |
| FeeSchedule | M47 | per election | Gazette notice |
| StatutoryNoticeRule | M52 | ~40 | Election Act schedule |
| Language codes | P04/M37 | 26 (all Kenyan languages) | ISO 639-3 |
| CountyMoU templates | M56 | 47 | Administrative |

---

## 7. Data Governance

### 7.1 Ownership and stewardship

Each entity has exactly one **owner module**. The owner module is the only service permitted to write to the entity's primary table. Other modules may read through the owner's API, via a Kafka-projected read-model, or via a shared read-only replica. This is enforced at the service mesh layer (OPA + Linkerd policies).

### 7.2 Schema registry

All Kafka event schemas are registered in Confluent Schema Registry (hosted on-prem). Schema evolution follows:
- Backward-compatible changes (add optional field): allowed at any time.
- Forward-compatible changes: require dual-approval change record.
- Breaking changes: new major version topic; old topic supported for ≥ 6 months.

Schema IDs are embedded in CloudEvents envelope (`dataschema` field). Consumers that receive an unrecognised schema version must reject the message, log it, and alert.

### 7.3 Versioning

Every mutable entity carries a `version` integer (optimistic concurrency). On conflict, the write is rejected with `409 Conflict`. Event-sourced entities never update in place; the version counter on the aggregate root tracks the event sequence number.

### 7.4 Deprecation policy

Entities deprecated after the pilot election phase require:
1. Six-month notice in the API changelog.
2. Sunset annotation in the OpenAPI spec.
3. Dual-commissioner sign-off on final removal.
4. Archived copy in M69 before deletion from operational tables.

---

## 8. Compliance Mapping

### 8.1 DPA 2019 §26 — Lawful basis per entity

| Entity | Lawful basis | Purpose |
|---|---|---|
| Voter | Explicit consent + public task (Elections Act §44) | Voter registration |
| BiometricTemplate | Explicit consent (DPA 2019 §25 — biometric category) | Identity deduplication |
| IDDocumentScan | Explicit consent | Identity verification |
| VoterSignature | Explicit consent | Authentication |
| VPRSApplication | Legitimate interest (pre-registration) | Efficient registration |
| User | Contract (employment) | System administration |
| SubjectAccessRequest | Legal obligation (DPA 2019 §26(1)(c)) | Data subject rights |
| MediaAccreditation | Public task | Press freedom + election transparency |
| ObserverOrganisation | Public task | International election observation |
| StaffApplication / PayrollRun | Contract | Employment |
| MedicalIncident | Vital interests | Emergency response |

### 8.2 Elections Act evidentiary retention (permanent)

- Form34A, Form34B, Form34C (and 35-38 series)
- ResultsPublicationSnapshot
- Declaration
- CertificateOfElection
- ReproducibilityBundle
- MerkleLogRoot
- KitCustodyEvent
- AuditEvent (archive tier)
- RegisterSnapshot

### 8.3 Constitution Art. 138(2) ballot secrecy

Entities designed to guarantee ballot secrecy:
- **VoterIdentificationEvent**: voter_id recorded but no link to ballot. Zero-knowledge property: knowing a voter authenticated tells you nothing about how they voted.
- **ParticipationToken**: issued to voter at kit; token hash in public bundle, but no voter-id link is stored anywhere.
- **PreMarkValidationEvent**: station-aggregate only; no voter_id field.
- **Form34A**: candidate vote counts only; no voter attribution possible.

### 8.4 15-year retention (per `expect.md` and Elections Act evidentiary)

- AuditEvent (archive tier)
- GrievanceCase
- EvidenceBundle
- TransmissionEvent
- CandidateDisclosure
- ElectionBudget / BudgetLine / PaymentVoucher
- PayrollRun / PayrollLine
- RedTeamEngagement / SecrecyFinding
- ArchiveCollection (minimum, per DataSource.retention_years)

---

## 9. Reconciliation Entities

The following canonical entities bridge multiple modules that independently reference the same concept. These are the single-source-of-truth records:

| Canonical Entity | Bridges | Notes |
|---|---|---|
| **Voter** (M02) | M01, M03, M04, M05, M06, M07, M12, M13, M35, M36, M53, M64 | Every module that needs voter identity references Voter.voter_id from M02. No module may maintain a duplicate voter identity store. |
| **PollingStation** (M18) | M01, M05, M06, M08, M19, M20, M24, M25, M27, M31, M35, M44, M63 | Boundary of truth for station codes and geo-fence polygons. Kit provisioning, result forms, queue monitoring, and ballot chain all reference the same station record. |
| **Kit** (M20) | M01, M05, M08, M16, M32, M33 | Every telemetry, custody, and configuration event references M20's Kit record by kit_id. |
| **Election** (M52) | All modules | Single election record drives the state machine that all modules check before processing operations. |
| **GazetteNotice** (M52) | M12, M35, M39, M45, M48, M49, M50, M51, M64 | Statutory authority for any gazettable action. Modules must reference a GazetteNotice before executing constrained operations (register close, candidate gazette, ballot print order). |
| **AuditEvent** (P05) | All modules | Every state change in the system emits an AuditEvent. The P05 audit service is the single consumer that validates signatures, appends to the Merkle log, and archives. |
| **User** (P04) | All modules | Single system user identity. Roles and permissions are attached to User. No module may define local user accounts. |

---

## 10. Gaps

The following entity references appear in module specifications but lack full canonical definitions in this version. They are flagged for resolution at Preliminary Project Plan phase:

| Gap | Referenced in | Notes |
|---|---|---|
| `MedicalUnit` / `MedicalIncident` | M60 | Defined but ownership and integration with NDMU external system unclear |
| `CountyDataUpload` / `CrossJurisdictionIncident` | M56 | County coordination module; data model sketched, owner interface with County governments needs clarification |
| `StatutoryReport` / `TestimonyWorkspace` | M58 | Parliamentary/Judiciary reporting module; format depends on output of Judiciary CTS integration |
| `AuditEngagement` / `AuditQuery` | M59 | Auditor-General interface; the AG's own data schema needs alignment |
| `ForecastRelease` | M44 | Consumption of live CVR data in real time needs explicit ACL from M02 |
| `RTSMetadataAnalysisRun` | M61 | Secrecy assurance testing; overlaps with M27 PreMarkValidationEvent — canonical owner needs resolution |
| `JudiciaryHandoff` | M58 | Depends on Judiciary CTS API contract not yet finalised |
| Diaspora ballot physical return receipts | M65 | Physical Post / Container chain defined; reconciliation with DiasporaForm34AD not fully wired |
| `FRCCashFlowEstimate` | M66 | Depends on FRC API data format; provisional schema |
| Mobile app local state (M29-M34) | M29-M34 | Mobile apps maintain encrypted local SQLite state; full schema deferred to Technical Design phase |

---

## 11. Immutability Guarantees

**Write-once immutable** (once committed, no field ever changes):
- AuditEvent, MerkleLogRoot
- Form34A (image + hash + signature)
- Form34B, 34C, 35-38 series (signed versions)
- ResultsPublicationSnapshot, ReproducibilityBundle
- KitCertificate (once issued), TamperSeal (once applied)
- KitCustodyEvent, VoterIdentificationEvent
- PrivilegedSession, Declaration, CertificateOfElection
- PrintOrder (once countersigned), SerialRangeAssignment

**Versioned-mutable** (every change writes a new version row):
- Voter, PollingStation, Candidate, Kit, User, PoliticalParty, Boundary

**Soft-deletable with retention schedule**:
- VPRSApplication (anonymised after 12 months)
- SupportTicket (deleted after 5 years unless linked to permanent event)

---

## 12. Retention Schedule Summary

| Data class | Retention | Enforcement |
|---|---|---|
| Voter register + biometrics | Contract + 5 years | Retention scheduler + ODPC audit |
| Results (Forms 34-38 signed) | Permanent | Constitutional archive + M69 |
| AuditEvent online | 90 days | ClickHouse TTL |
| AuditEvent archive | 15 years (then permanent for election results) | Object store + escrow + M69 |
| Declaration / Certificate of Election | Permanent | M69 |
| PrivilegedSession recording | 5 years | Dual-custody encryption |
| VPRSApplication (unconverted) | 12 months, then anonymised | Scheduler |
| SupportTicket | 5 years | Scheduler |
| DPIA + BreachNotification + SAR | Permanent | Compliance requirement |
| Campaign finance records | 15 years | ECFA 2013 |
| Election budget / payroll | 15 years | Public Finance Management Act |
| Grievance case | 15 years | Elections Act |
| Sandbox / test data | Session lifetime | Synthetic-only guarantee |

---

## 13. Cross-References

- Schema DDL (PostgreSQL) delivered as separate artefact at `/sw/db/` at project execution.
- Event schemas (CloudEvents + JSON payloads) in `API-CATALOG.md` §7.
- Entity-relationship diagrams in `../diagrams/architectural/a4-domain-er.svg`.
- Biometric format standards: ISO/IEC 19794-2/5/6, ANSI/NIST-ITL 1-2011, WSQ, JPEG-2000.
