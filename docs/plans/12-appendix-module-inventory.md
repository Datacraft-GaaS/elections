# Appendix A — Module inventory (condensed)

Size key: S <2 dev-wk, M 2-6, L 6-12, XL >12. Home service per ADR-0020.

| M | slug | svc | surfaces | size | entities (core) | key deps | notable risk |
|---|---|---|---|---|---|---|---|
| 01 | bvr-device | biometric-edge | kit, legacy-android | L | Voter, BiometricTemplate, IDDocScan, KitAttestation | M02 M03 P01 P04 | sensor SDK coupling; ISO/FBI cert |
| 02 | bvr-backend | bvr | api, admin, batch | XL | Voter(versioned), Template, AuditEvent, MerkleRoot | M01 M03 IPRS HSM | 30M scale, 99.99%, IPRS variability |
| 03 | abis | abis | api, batch, admin | XL | TemplateIndex, MatchRequest, AdjudicationCase, EnclaveAttestation | M02 TDX/SEV HSM | vendor-vs-build; enclave attestation |
| 04 | vprs | vprs | web-public, PWA, api | M | VPRSApplication | IPRS M01 M02 | IPRS latency; enumeration abuse |
| 05 | bvi-device | biometric-edge | kit, legacy-android | L | PollingSession, IdentificationEvent, RegisterExtract | M02 M06 P01 P04 | GNSS spoof; legacy parity |
| 06 | bvi-backend | bvi | api, admin, public-feed | L | KitConfigBundle, IdentificationEvent, TurnoutRollup, KitCert | M02 M05 HSM | 42M events/day; 60s revocation |
| 07 | bvv | bvi | web-public, kiosk, api, admin | M | VoterLookupAudit, Objection | M02 | enumeration vs DPA |
| 08 | rts-device | rts | kit | L | FormScan, TransmissionAttempt, ParticipationToken | M10 telco HSM | SMS compression of signed forms |
| 09 | rts-ro | rts | desktop, api | M | Form34A/B/C, ICRExtraction, DisputeCase | M08 M10 HSM | dispute UX; session recording |
| 10 | rts-backend | rts | api, batch, ml | XL | Form34*, PublicationSnapshot, MerkleRoot | M08 M09 HSM P05 | TLA+ proof; ICR handwriting |
| 11 | public-portal | portal | web-public, api | L | PublishedForm, MerkleRoot, InclusionProof | M10 M06 TL | petition-window peak; cache poisoning |
| 12 | crms | crms | admin, public, api | M | Candidate, Nomination, ClearanceDecision, BallotProof | M02 HSM KRA EACC | quota rules; printer PDF fidelity |
| 13 | public-verifier-app | field-mobile | mobile, web, USSD, WA, api | XL | VerificationQuery, ParticipationTokenQuery, Grievance | M10 M11 M14 M16 M17 M19 M23 | WA quota; clone apps |
| 14 | reproducibility-bundle | transparency | cli, web, api, batch | L | Bundle, ParticipationToken, BugBounty | M08 M10 HSM | cross-arch determinism; Gazette/KBC |
| 15 | iab-dashboard | ecc | admin, api | M | IABEscalation, Complaint, Report | P05 P06 M14 M20 M26 | hw-key logistics |
| 16 | election-command-centre | ecc | admin(multi-role), api, mobile | XL | KitTelemetrySnapshot, ECCAlert, S44AEvent, Dispatch | M08 M10 M13 M17-20 M26 | RBAC complexity; NIS loop |
| 17 | speech-integrity | ai-platform | analyst-admin, ml, api | XL | ContentFlag, SafetyIncident, EvidencePackage | M13 M16 M15 platforms NCIC | API access; Kenyan-language bias |
| 18 | boundary-transparency | boundary | web-public, api, batch | M | Boundary, BoundaryChange, FairnessMetric, GeofencePolygon | M02 HSM NLC KNBS | shapefile currency |
| 19 | queue-transparency | queue | api, public-map, ECC | M | QueueSnapshot, DisenfranchisementFlag, S14ARecommendation | M08 M13 M16 M23 | signal fusion; CV headcount |
| 20 | inventory-management | inventory | admin, api, mobile, batch | XL | Asset, CustodyEvent, Seal, Consumable, ReservePool | M08 M18 HSM MES logistics | OEM/logistics APIs; RFID lead time |
| 21 | rla-audit-tool | governance | api, desktop(offline), web, cli | M | AuditRun, SampledBallot, EntropySource | M10 M11 P05 P06 NIST | legal clarity on recount trigger |
| 22 | campaign-finance | transparency | admin, party-portal, public, api | M | CandidateDisclosure, CashAggregate, EvidencePackage | M12 M23 CBK/FRC | CBK/FRC MoU |
| 23 | grievance-channels | ecc | public, api, mobile, kiosk, webhooks | L | GrievanceCase, EvidencePackage, WhistleblowerProfile | WA USSD SMS M08 M10 M22 M26 M28 | WA approval 6-8wk |
| 24 | diaspora-voting | biometric-edge | kit, api, admin, public | L | EmbassyStation, DiasporaRoll, Form34AD, PouchCustody | M01 M02 M05 M06 M03 M10 | kit import refusal |
| 25 | special-voter-workflows | support-admin | kit, api, admin, public | L | SpecialVoterFacility, SpecialVoterEvent, IDPReassignment | M01 M02 M05 M06 M10 | MoU refusal; s.44A misuse |
| 26 | anomaly-fraud-detection | ai-platform | kit(L1), api, ml, ECC, public | XL | DetectionAlert, TelemetryPacket, CrossSourceComparison | M21 M23 M08 M10 M11 | alert fatigue; canary messaging |
| 27 | intent-preservation | support-admin | kit, api, admin, public | L | PreMarkValidationEvent, SpoiltBallotEvent, AdjudicationEvent | M08 M10 M26 M28 | coercion perception |
| 28 | ai-monitoring-stack | ai-platform | kit, api, ml, registry, cli | XL | AIModel, AIDecisionLog, ModelCard, PIIStrippingEvent | M01 M05 M03 M10 M23 M26 M27 | deepfake evasion; PII-strip regressions |
| 29 | mobile-citizen-verify | field-mobile | mobile(A/iOS/PWA), api | L | local caches, bookmarks, verification records | M04 M07 M10 M11 M13 M14 | KES-10k Android; store approval |
| 30 | mobile-party-agent | field-mobile | mobile(MDM), api | M | AgentCountersignature | M06 M08 M09 M10 M11 HSM | attestation false-fail; duress UX |
| 31 | mobile-poll-clerk | field-mobile | mobile(MDM), api | M | incident, EOD report | M05 M06 M07 M02 M20 | M06 companion API |
| 32 | mobile-field-support | field-mobile | mobile(MDM), api | M | FieldSupportTicket, diag logs | M20 M06 M16 O01 O02 O04 | remote-action safety |
| 33 | mobile-personnel-dispatch | field-mobile | mobile(public+MDM), api | XL | PersonnelAttendanceRecord | O05 M06 M16 M20 payroll | 250K concurrent; GPS spoof |
| 34 | mobile-observer | field-mobile | mobile(MDM), api | M | ObserverVisitLog, verification record | M10 M11 M13 M21 AG | AG mirror undefined |
| 35 | cvr-continuous-register | bvr | api, batch, admin, public | L | VoterRegisterEvent, RegisterSnapshot, BatchIngestionReport | M02 M03 M36 CRS NRB KRA | MoU delay; *400# abuse |
| 36 | voter-deregistration-appeal | bvr | api, admin, public, USSD, cli | M | DeregistrationCase, AppealRecord | M35 M02 M03 M13 ODPC | adjudicator scaling |
| 37 | voter-education-cms | training | api, admin, public, M13 feed | M | ContentItem, ContentVersion, FactCheckReview | M13 M40 M11 M28 WA CDN | fact-check throughput |
| 38 | poll-worker-training-lms | training | api, admin, mobile(offline), QR | L | CourseModule, WorkerEnrollment, WorkerCertificate | M27 M33 M37 HSM HR | remote connectivity |
| 39 | ballot-printing-chain | inventory | api, admin, kit(RFID), public | L | PrintOrder, LotData, CustodyEvent, StationReconciliation | M08 M06 M16 M40 MES KRA | printer MES API |
| 40 | election-court-it-interface | governance | api, admin, public-index | L | BundleItem, EvidenceBundle, PetitionerAccess, TechnicalWitness | 11 upstream modules, eCourt | upstream item delay |
| 41 | post-election-review | governance | api, admin, public, M13 | M | Finding, Recommendation, ParliamentReferral, LongDripAlert | M13 M11 M15 M28 | silent scheduler |
| 42 | media-accreditation | training | api, admin, public, scanner | S/M | MediaAccreditation, MediaOrganisation | MCK M16 M43 | MCK API; offline CRL |
| 43 | observer-accreditation | training | api, admin, public, scanner | M | ObserverOrganisation, IndividualObserver | M15 M34 M42 M16 | late rosters |
| 44 | turnout-forecasting | ai-platform | api, ml, batch, public-dash, admin | M | ForecastRelease, HourlyTurnoutReport, VarianceAlert | M02 M08 M16 M41 M50 KMD | PO skips reports |
| 45 | party-registration | crms | api, admin, public, cli | M | PoliticalParty, CoalitionRegistration, FactionDispute | M02 M12 M47 M67 PPDT | CVR extract staleness |
| 46 | party-primaries-oversight | crms | kit(lite), api, admin, public, portal | L | PrimaryOptIn, PrimaryResult, PrimaryDispute | M45 M12 M44 M16 M20 M43 | low opt-in |
| 47 | nomination-fee-payment | crms | public, admin, api | M | FeeSchedule, Payment, Waiver, Refund | M12 M45 Daraja bank | duplicate callback |
| 48 | recall | governance | public, admin, api, kit(BVI recall) | L | RecallPetition, RecallElection, SignatoryRecord | M02 M05/06 M08 M50 | Art.104(3) interpretation |
| 49 | referendum-mechanics | governance | public, admin, api, kit | L | Referendum, CountyResult, NationalResult | M08 M10 M11 M51 M52 | county threshold off-by-one; TLA+ |
| 50 | by-election-operations | governance | admin, api, kit, mobile | L | ByElection, CostReport, RegisterSubset | M04 M10 M12 M20 M22 M51 M52 M55 | <21-day notice |
| 51 | result-declaration | rts | admin, api | L | Declaration, CertificateOfElection, PetitionWindowTracker | M10 M52 M40 M58 HSM | run-off logic; immutability vs correction |
| 52 | election-calendar-gazette | portal | public, admin, api | M | CalendarEvent, GazetteNotice, StatutoryRule | GovPrinter M51 M49 M50 M18 | GP API |
| 53 | voter-id-card | bvr | public, admin, api, kit | XL (flagged) | VoterCard, CardBatch, CollectionRecord | NRB manufacturer M06 M04 | scope activation |
| 54 | budget-expenditure | support-admin | admin, api | L | Budget, Procurement, PaymentVoucher, CoBReport | IFMIS PPRA M47 M55 M59 | IFMIS friction |
| 55 | hr-payroll | support-admin | public, admin, api, mobile | XL | JobApplication, StaffContract, PayrollRun, StatutoryDeduction | M-Pesa B2C KRA NSSF NHIF M38 M33 M54 | B2C limits; SoD |
| 56 | county-national-coord | governance | public(county), admin, api | M | MoU, CountyDataset, MinistryApproval, CoordinationIncident | IDMS SAML M16 M18 M33 | heterogeneous IdPs |
| 57 | intl-observer-coord | governance | public, admin, api | M | ObserverMission, Briefing, ObservationReport, JointStatement | M15 M43 M11 | workspace isolation |
| 58 | parliamentary-judiciary-reporting | governance | admin, api | M | StatutoryReport, Testimony, JudiciaryHandover | M40 M51 M10 | draft versioning |
| 59 | oag-interface | governance | admin, api | M | AuditEngagement, EvidenceScope, EvidencePackage | M54 M14 M20 M55 M69 | scope creep |
| 60 | medical-first-response | training | admin, mobile(M31), api | M | MedicalIncident, DispatchRequest, ClusterAlert | KRCS MoH NDMU M16 M31 | no PHI allowed |
| 61 | secrecy-assurance-testing | ai-platform | admin, api | M | RedTeamEngagement, Finding, IABAttestation | staging clones, M15 | isolation breach |
| 62 | transmission-legal-audit | governance | admin, api, party-portal | M | TransmissionEvent, ComplianceVerdict, LegalRule | M10 M40 P05 | rule drift |
| 63 | ballot-serial-chain | rts | admin, mobile(M31), api, kit | L | SerialAssignment, IssuanceReceipt, SpoiltBallotEvent, Reconciliation | M39 M20 M31 M08 M10 | false result-holds |
| 64 | voter-list-inspection | portal | public, admin, api | M | RollSnapshot, Objection, CorrectionClaim, AutoReferralPacket | M04 M01 M10 eCourt | PII enumeration; T-60 race |
| 65 | diaspora-logistics | inventory | mobile(offline), admin, api | L | BallotContainer, CustodyEvent, PreClearanceManifest | M24 M20 M16 MFA KRA airlines | embargo/customs |
| 66 | financial-forensics | governance | public, admin, api, batch, ml | L | AnalysisCycle, EstimateRecord, ReferralPacket, ReplicationPackage | M22 M10 M69 FRC KNBS | method contested |
| 67 | party-symbol-registry | crms | public, admin, api | M | SymbolSubmission, ColourRegistration, BallotDesignPackage | M45 M12 printer Pantone | near-threshold disputes |
| 68 | referendum-explainer | portal | public, mobile, api, batch(TTS) | L | ReferendumQuestion, ExplainerContent, FactCheckAnnotation | M49 M17 M11 M29 M23 | WCAG AAA; translation |
| 69 | long-term-archive | support-admin | public, admin, api, batch | L | ArchiveCollection, IngestJob, PreservationAttestation, DestructionCertificate | M14 M10 M04 M20 M22 M63 M66 KNADS DataCite | format migration |

Integrations: I01 IPRS · I02 MRZ · I03 telco VPN · I04 Power BI · I05 legacy ABIS · I06 legacy Android shim (~14,100) · I07 HSM PKCS#11 · I08 SMS/USSD · I09 ODPC breach · I10 KEBS/OEM warranty · I11 WhatsApp · I12 KRA tax · I13 IPFS · I14 satellite; plus M-Pesa C2B/B2C, IFMIS, KRA iTax, NSSF, NHIF/SHIF, Gov Printer, KBC SFTP, KNADS, DataCite, eCourt, PPDT, MCK, NCIC/DCI/ODPP, CBK/FRC, KMD, Kenya Airways/IATA, KRA e-Clearance, Pantone, social platforms, LiteLLM, NLLB, CRS, NRB, EACC, KPS/KDF/UNHCR, printer MES, card manufacturer.

SLAs: availability election-critical 99.99% (M01,02,03,05,06,08,09,10,27,49,50,51,63), cycle-critical 99.9%, routine 99.5%; BVI 1:1 p95 800 ms / p99 1.5 s @ 18k req/s, bvi-svc p99 ≤100 ms; RTS submit p95 500 ms / p99 1 s @ 60k concurrent, ingest p99 ≤2 s, RTO ≤30 min RPO ≤5 min; ABIS 1:N p95 5 s / p99 8 s, 10k enrolments/hr, FMR ≤0.001% FNMR ≤0.1%; portal p95 200 ms / p99 500 ms @ 5M/hr, proofs ≤30 s; BVR ingest 1k rec/s, 30M gen ≤30 min, export ≤4 h, RTO ≤4 h RPO ≤15 min; Kafka 100k ev/s; kit boot ≤90 s, ≥12 enrolments/hr, 7-day queue, scan+sign+transmit ≤90 s, SMS fallback ≤10 min; M33 250k/90 min; M55 payroll 250k ≤4 h; M13 10M concurrent p99 <500 ms; WCAG 2.2 AA (AAA M68); ASVS L3 election-critical; SBOM ≤1 h; 11 languages.

