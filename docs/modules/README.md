# Modules index: 69 modules

Slugs match the upstream Datacraft spec (`work_docs/01-Projects/IEBC/sw/modules/M<nn>-<slug>.md`) with the M-number prefix dropped. M-numbers are preserved here for traceability.

Each module has a design/spec home under `modules/<slug>/docs/SPEC.md`. Implementation lives in the module's home K8s service under `services/<svc>/src/` (see ADR-0020 for the 18-service grouping).

| M# | slug | home service | notes |
|---|---|---|---|
| M01 | bvr-device | biometric-edge-svc | KIEMS kit biometric enrolment (Flutter) |
| M02 | bvr-backend | bvr-svc | voter register backend |
| M03 | abis | abis-svc | 1:N dedup, confidential compute (SEV-SNP/TDX) |
| M04 | vprs | vprs-svc | public self-preregistration |
| M05 | bvi-device | biometric-edge-svc | KIEMS kit voter identification (Flutter) |
| M06 | bvi-backend | bvi-svc | turnout aggregation, station bind |
| M07 | bvv | bvi-svc | register-inspection portal |
| M08 | rts-device | rts-svc | Form 34A capture on kit (Flutter) |
| M09 | rts-ro | rts-svc | constituency RO tally (Tauri desktop) |
| M10 | rts-backend | rts-svc | national results aggregation |
| M11 | public-portal | portal-svc | CDN-first signed results (Astro + LiveView) |
| M12 | crms | crms-svc | candidate registration + party portal (Django) |
| M13 | public-verifier-app | field-mobile-svc | mobile + PWA + SMS/USSD/WhatsApp verifier |
| M14 | reproducibility-bundle | transparency-svc | `make reproduce` toolkit |
| M15 | iab-dashboard | ecc-svc | Independent Assurance Board console (LiveView) |
| M16 | election-command-centre | ecc-svc | E-SOC/COC/CAOC operator workstation (Tauri hosting LiveView) |
| M17 | speech-integrity | ai-platform-svc | hate-speech / CIB / deepfake detection (Python ML) |
| M18 | boundary-transparency | boundary-svc | boundary explorer/simulator/geofence |
| M19 | queue-transparency | queue-svc | real-time queue map (LiveView) |
| M20 | inventory-management | inventory-svc | RFID+GPS Merkle chain of custody |
| M21 | rla-audit-tool | governance-svc | SHANGRLA post-election audit (Python) |
| M22 | campaign-finance-transparency | transparency-svc | ECFA spending with CBK/FRC integration |
| M23 | grievance-channels | ecc-svc | multi-channel intake (LiveView) |
| M24 | diaspora-voting | biometric-edge-svc | embassy-hosted kits, delayed-batch |
| M25 | special-voter-workflows | support-admin-svc | prisoners/hospital/KDF/IDP modes |
| M26 | anomaly-fraud-detection | ai-platform-svc | 7-layer detection stack (Python ML) |
| M27 | intent-preservation | support-admin-svc | pre-mark validation, spoilage workflow |
| M28 | ai-monitoring-stack | ai-platform-svc | governed AI model registry, on-device INT8 |
| M29 | mobile-citizen-verify | field-mobile-svc | native IEBC Verify (Flutter) |
| M30 | mobile-party-agent | field-mobile-svc | party agent countersignature (Flutter) |
| M31 | mobile-poll-clerk | field-mobile-svc | deputy PO / kiosk companion (Flutter) |
| M32 | mobile-field-support | field-mobile-svc | field engineer app (Flutter) |
| M33 | mobile-personnel-dispatch | field-mobile-svc | GPS check-in/out for ~46k stations (Flutter) |
| M34 | mobile-observer | field-mobile-svc | accredited observer app (Flutter) |
| M35 | cvr-continuous-voter-register | bvr-svc | continuous voter register lifecycle |
| M36 | voter-deregistration-appeal | bvr-svc | deregistration + appeal workflow |
| M37 | voter-education-cms | training-svc | voter education CMS (Django) |
| M38 | poll-worker-training-lms | training-svc | poll worker LMS (Django) |
| M39 | ballot-printing-chain | inventory-svc | ballot print custody chain |
| M40 | election-court-it-interface | governance-svc | court IT interface |
| M41 | post-election-review | governance-svc | post-election review workflow |
| M42 | media-accreditation-portal | training-svc | media accreditation (Django admin + Astro public) |
| M43 | observer-accreditation-portal | training-svc | observer accreditation (Django admin + Astro public) |
| M44 | turnout-forecasting-analytics | ai-platform-svc | turnout forecasting (Python) |
| M45 | party-registration-management | crms-svc | party registration lifecycle |
| M46 | party-primaries-oversight | crms-svc | primaries oversight |
| M47 | nomination-fee-payment | crms-svc | nomination fee payment |
| M48 | recall | governance-svc | recall petition workflow |
| M49 | referendum-mechanics | governance-svc | referendum mechanics |
| M50 | by-election-operations | governance-svc | by-election operations |
| M51 | result-declaration-certification | rts-svc | result declaration (event-sourced) |
| M52 | election-calendar-gazette | portal-svc | calendar + gazette publication (Astro) |
| M53 | voter-id-card | bvr-svc | voter ID card issuance |
| M54 | budget-expenditure | support-admin-svc | budget + expenditure |
| M55 | hr-payroll | support-admin-svc | HR + payroll |
| M56 | county-national-coordination | governance-svc | county-national coordination |
| M57 | international-observer-coordination | governance-svc | international observer coordination |
| M58 | parliamentary-judiciary-reporting | governance-svc | parliamentary + judiciary reporting |
| M59 | oag-interface | governance-svc | OAG interface |
| M60 | medical-first-response | training-svc | medical first-response |
| M61 | secrecy-assurance-testing | ai-platform-svc | secrecy assurance testing |
| M62 | transmission-legal-basis-audit | governance-svc | transmission legal-basis audit (audit SDK 100% coverage) |
| M63 | ballot-serial-chain | rts-svc | ballot serial chain (event-sourced) |
| M64 | voter-list-public-inspection | portal-svc | voter-list public inspection (Astro) |
| M65 | diaspora-ballot-physical-logistics | inventory-svc | diaspora ballot physical logistics |
| M66 | election-financial-forensics | governance-svc | financial forensics |
| M67 | party-symbol-colour-registry | crms-svc | party symbol/colour registry |
| M68 | referendum-explainer | portal-svc | referendum explainer (Astro + LiveView) |
| M69 | long-term-archive | support-admin-svc | long-term archive (BagIt) |

## Event-sourced modules (only these 4 per ADR-0013)

M08/M09/M10 (rts-svc), M20 (inventory-svc), M51 (rts-svc), M63 (rts-svc). Everything else uses state + immutable audit shadow table.

## Owners

TBD. Owner + Slack channel goes in each module's `docs/SPEC.md` metadata.

## Traceability

- Upstream spec: `work_docs/01-Projects/IEBC/sw/modules/M<nn>-<slug>.md`
- Service topology: ADR-0020
- Event-source scope: ADR-0013
- Data class (per column/field): ADR-0016 + `packages/data-dictionary/`
