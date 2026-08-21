# Module Index

69 modules across 18 services. Each row links to the detailed module specification in `docs/plans/modules/`. Do not duplicate content from those files; link to them.

Legend: M-ID = module number; svc = home service; tests = primary test file path pattern (relative to repo root); EC = edge-case count in `docs/plans/08-edge-case-matrix.md`.

| M-ID | Slug | Service | Module spec | Primary tests | EC rows | Statutory citations |
|---|---|---|---|---|---|---|
| M01 | bvr-device | biometric-edge | [docs/plans/modules/bvr-device.md](../plans/modules/bvr-device.md) | `services/biometric-edge-svc/tests/ci/test_bvr_device.py` | 12 | Elections Act s.6, ISO 19794-2 |
| M02 | bvr-backend | bvr | [docs/plans/modules/bvr-backend.md](../plans/modules/bvr-backend.md) | `services/bvr-svc/tests/ci/test_bvr_backend.py` | 18 | Elections Act s.6, s.7, DPA 2019 s.25 |
| M03 | abis | abis | [docs/plans/modules/abis.md](../plans/modules/abis.md) | `services/abis-svc/tests/ci/test_abis.py` | 15 | Elections Act s.6, ISO/IEC 30107-3 |
| M04 | vprs | vprs | [docs/plans/modules/bvi-backend.md](../plans/modules/bvr-backend.md) | `services/vprs-svc/tests/ci/test_vprs.py` | 8 | Elections Act s.5, s.8 |
| M05 | bvi-device | biometric-edge | [docs/plans/modules/bvi-device.md](../plans/modules/bvi-device.md) | `services/biometric-edge-svc/tests/ci/test_bvi_device.py` | 14 | Elections Act s.44, s.44A |
| M06 | bvi-backend | bvi | [docs/plans/modules/bvi-backend.md](../plans/modules/bvi-backend.md) | `services/bvi-svc/tests/ci/test_bvi_backend.py` | 16 | Elections Act s.44, s.44A |
| M07 | bvv | bvi | [docs/plans/modules/bvi-backend.md](../plans/modules/bvi-backend.md) | `services/bvi-svc/tests/ci/test_bvv.py` | 6 | DPA 2019 s.25 |
| M08 | rts-device | rts | [docs/plans/modules/rts-backend.md](../plans/modules/rts-backend.md) | `services/rts-svc/tests/ci/test_rts_device.py` | 20 | Elections Act s.74, s.83 |
| M09 | rts-ro | rts | [docs/plans/modules/rts-backend.md](../plans/modules/rts-backend.md) | `services/rts-svc/tests/ci/test_rts_ro.py` | 12 | Elections Act s.74, s.83 |
| M10 | rts-backend | rts | [docs/plans/modules/rts-backend.md](../plans/modules/rts-backend.md) | `services/rts-svc/tests/ci/test_rts_backend.py` | 25 | Elections Act s.74, s.83, s.85 |
| M11 | public-portal | portal | [docs/plans/modules/boundary-transparency.md](../plans/modules/boundary-transparency.md) | `services/portal-svc/tests/ci/test_public_portal.py` | 10 | Elections Act s.85, s.87 |
| M12 | crms | crms | [docs/plans/modules/crms.md](../plans/modules/crms.md) | `services/crms-svc/tests/ci/test_crms.py` | 14 | Elections Act ss.22-38, s.70 |
| M13 | public-verifier-app | field-mobile | [docs/plans/modules/boundary-transparency.md](../plans/modules/boundary-transparency.md) | `services/field-mobile-svc/tests/ci/test_public_verifier.py` | 11 | Elections Act s.87 |
| M14 | reproducibility-bundle | transparency | [docs/plans/modules/ballot-printing-chain.md](../plans/modules/ballot-printing-chain.md) | `services/transparency-svc/tests/ci/test_bundle.py` | 9 | Elections Act s.85, NIST SP 800-90B |
| M15 | iab-dashboard | ecc | [docs/plans/modules/iab-dashboard.md](../plans/modules/iab-dashboard.md) | `services/ecc-svc/tests/ci/test_iab.py` | 8 | Elections Act s.88A |
| M16 | election-command-centre | ecc | [docs/plans/modules/election-command-centre.md](../plans/modules/election-command-centre.md) | `services/ecc-svc/tests/ci/test_ecc.py` | 22 | Elections Act s.44A, s.14 |
| M17 | speech-integrity | ai-platform | [docs/plans/modules/ai-monitoring-stack.md](../plans/modules/ai-monitoring-stack.md) | `services/ai-platform-svc/tests/ci/test_speech_integrity.py` | 10 | Elections Act s.90, NCIC Act s.13 |
| M18 | boundary-transparency | boundary | [docs/plans/modules/boundary-transparency.md](../plans/modules/boundary-transparency.md) | `services/boundary-svc/tests/ci/test_boundary.py` | 7 | Constitution Art.89, IBRA Act s.26 |
| M19 | queue-transparency | queue | [docs/plans/modules/election-command-centre.md](../plans/modules/election-command-centre.md) | `services/queue-svc/tests/ci/test_queue.py` | 9 | Elections Act s.14 |
| M20 | inventory-management | inventory | [docs/plans/modules/inventory-management.md](../plans/modules/inventory-management.md) | `services/inventory-svc/tests/ci/test_inventory.py` | 18 | Elections Act s.44, s.45 |
| M21 | rla-audit-tool | governance | [docs/plans/modules/iab-dashboard.md](../plans/modules/iab-dashboard.md) | `services/governance-svc/tests/ci/test_rla.py` | 13 | Elections Act s.82, NIST SP 800-90B |
| M22 | campaign-finance | transparency | [docs/plans/modules/campaign-finance-transparency.md](../plans/modules/campaign-finance-transparency.md) | `services/transparency-svc/tests/ci/test_campaign_finance.py` | 9 | Elections Act s.90, PFM Act s.85 |
| M23 | grievance-channels | ecc | [docs/plans/modules/grievance-channels.md](../plans/modules/grievance-channels.md) | `services/ecc-svc/tests/ci/test_grievance.py` | 14 | Elections Act ss.74-76 |
| M24 | diaspora-voting | biometric-edge | [docs/plans/modules/diaspora-voting.md](../plans/modules/diaspora-voting.md) | `services/biometric-edge-svc/tests/ci/test_diaspora.py` | 11 | Elections Act s.55, Kenya Citizenship Act |
| M25 | special-voter-workflows | support-admin | [docs/plans/modules/intent-preservation.md](../plans/modules/intent-preservation.md) | `services/support-admin-svc/tests/ci/test_special_voter.py` | 10 | Elections Act s.44A, s.55 |
| M26 | anomaly-fraud-detection | ai-platform | [docs/plans/modules/anomaly-fraud-detection.md](../plans/modules/anomaly-fraud-detection.md) | `services/ai-platform-svc/tests/ci/test_anomaly.py` | 15 | Elections Act s.88A |
| M27 | intent-preservation | support-admin | [docs/plans/modules/intent-preservation.md](../plans/modules/intent-preservation.md) | `services/support-admin-svc/tests/ci/test_intent.py` | 12 | Elections Act s.72 |
| M28 | ai-monitoring-stack | ai-platform | [docs/plans/modules/ai-monitoring-stack.md](../plans/modules/ai-monitoring-stack.md) | `services/ai-platform-svc/tests/ci/test_ai_monitoring.py` | 8 | DPA 2019 s.25, AI governance policy |
| M29 | mobile-citizen-verify | field-mobile | [docs/plans/modules/boundary-transparency.md](../plans/modules/boundary-transparency.md) | `services/field-mobile-svc/tests/ci/test_mobile_citizen.py` | 7 | Elections Act s.87 |
| M30 | mobile-party-agent | field-mobile | [docs/plans/modules/ballot-printing-chain.md](../plans/modules/ballot-printing-chain.md) | `services/field-mobile-svc/tests/ci/test_party_agent.py` | 9 | Elections Act s.71, s.74 |
| M31 | mobile-poll-clerk | field-mobile | [docs/plans/modules/inventory-management.md](../plans/modules/inventory-management.md) | `services/field-mobile-svc/tests/ci/test_poll_clerk.py` | 8 | Elections Act s.44 |
| M32 | mobile-field-support | field-mobile | [docs/plans/modules/inventory-management.md](../plans/modules/inventory-management.md) | `services/field-mobile-svc/tests/ci/test_field_support.py` | 5 | Internal operational policy |
| M33 | mobile-personnel-dispatch | field-mobile | [docs/plans/modules/hr-payroll.md](../plans/modules/hr-payroll.md) | `services/field-mobile-svc/tests/ci/test_dispatch.py` | 10 | Elections Act s.33, Employment Act s.27 |
| M34 | mobile-observer | field-mobile | [docs/plans/modules/international-observer-coordination.md](../plans/modules/international-observer-coordination.md) | `services/field-mobile-svc/tests/ci/test_observer.py` | 6 | Elections Act s.34, s.35 |
| M35 | cvr-continuous-register | bvr | [docs/plans/modules/cvr-continuous-voter-register.md](../plans/modules/cvr-continuous-voter-register.md) | `services/bvr-svc/tests/ci/test_cvr.py` | 11 | Elections Act s.5, s.7 |
| M36 | voter-deregistration-appeal | bvr | [docs/plans/modules/cvr-continuous-voter-register.md](../plans/modules/cvr-continuous-voter-register.md) | `services/bvr-svc/tests/ci/test_dereg.py` | 8 | Elections Act s.10 |
| M37 | voter-education-cms | training | [docs/plans/modules/boundary-transparency.md](../plans/modules/boundary-transparency.md) | `services/training-svc/tests/ci/test_cms.py` | 7 | Elections Act s.42, s.44B |
| M38 | poll-worker-training-lms | training | [docs/plans/modules/poll-worker-training-lms.md](../plans/modules/poll-worker-training-lms.md) | `services/training-svc/tests/ci/test_lms.py` | 10 | Elections Act s.33, s.44 |
| M39 | ballot-printing-chain | inventory | [docs/plans/modules/ballot-printing-chain.md](../plans/modules/ballot-printing-chain.md) | `services/inventory-svc/tests/ci/test_printing.py` | 12 | Elections Act s.70, s.72 |
| M40 | election-court-it-interface | governance | [docs/plans/modules/election-court-it-interface.md](../plans/modules/election-court-it-interface.md) | `services/governance-svc/tests/ci/test_court.py` | 14 | Elections Act s.75, s.82 |
| M41 | post-election-review | governance | [docs/plans/modules/governance-review.md](../plans/modules/governance-review.md) | `services/governance-svc/tests/ci/test_review.py` | 9 | Elections Act s.87 |
| M42 | media-accreditation | training | [docs/plans/modules/media-accreditation.md](../plans/modules/media-accreditation.md) | `services/training-svc/tests/ci/test_media_accred.py` | 7 | MCK Act, Elections Act s.34 |
| M43 | observer-accreditation | training | [docs/plans/modules/international-observer-coordination.md](../plans/modules/international-observer-coordination.md) | `services/training-svc/tests/ci/test_observer_accred.py` | 8 | Elections Act s.34, s.35 |
| M44 | turnout-forecasting | ai-platform | [docs/plans/modules/anomaly-fraud-detection.md](../plans/modules/anomaly-fraud-detection.md) | `services/ai-platform-svc/tests/ci/test_forecast.py` | 6 | Internal reporting policy |
| M45 | party-registration | crms | [docs/plans/modules/crms.md](../plans/modules/crms.md) | `services/crms-svc/tests/ci/test_party.py` | 9 | Political Parties Act s.7, s.10 |
| M46 | party-primaries-oversight | crms | [docs/plans/modules/crms.md](../plans/modules/crms.md) | `services/crms-svc/tests/ci/test_primaries.py` | 10 | Political Parties Act s.21 |
| M47 | nomination-fee-payment | crms | [docs/plans/modules/crms.md](../plans/modules/crms.md) | `services/crms-svc/tests/ci/test_fees.py` | 7 | Elections Act s.22, s.23 |
| M48 | recall | governance | [docs/plans/modules/by-election-operations.md](../plans/modules/by-election-operations.md) | `services/governance-svc/tests/ci/test_recall.py` | 11 | Constitution Art.104 |
| M49 | referendum-mechanics | governance | [docs/plans/modules/by-election-operations.md](../plans/modules/by-election-operations.md) | `services/governance-svc/tests/ci/test_referendum.py` | 12 | Constitution Art.255-257, Referendum Act |
| M50 | by-election-operations | governance | [docs/plans/modules/by-election-operations.md](../plans/modules/by-election-operations.md) | `services/governance-svc/tests/ci/test_by_election.py` | 10 | Elections Act s.11 |
| M51 | result-declaration | rts | [docs/plans/modules/rts-backend.md](../plans/modules/rts-backend.md) | `services/rts-svc/tests/ci/test_declaration.py` | 11 | Elections Act s.74, s.83, s.85 |
| M52 | election-calendar-gazette | portal | [docs/plans/modules/election-calendar-gazette.md](../plans/modules/election-calendar-gazette.md) | `services/portal-svc/tests/ci/test_gazette.py` | 8 | Elections Act s.11, Kenya Gazette |
| M53 | voter-id-card | bvr | [docs/plans/modules/cvr-continuous-voter-register.md](../plans/modules/cvr-continuous-voter-register.md) | `services/bvr-svc/tests/ci/test_voter_card.py` | 6 | Elections Act s.7 (scope activation required) |
| M54 | budget-expenditure | support-admin | [docs/plans/modules/budget-expenditure.md](../plans/modules/budget-expenditure.md) | `services/support-admin-svc/tests/ci/test_budget.py` | 10 | PFM Act, PPRA Act |
| M55 | hr-payroll | support-admin | [docs/plans/modules/hr-payroll.md](../plans/modules/hr-payroll.md) | `services/support-admin-svc/tests/ci/test_payroll.py` | 14 | Employment Act, Tax Procedures Act |
| M56 | county-national-coord | governance | [docs/plans/modules/county-national-coordination.md](../plans/modules/county-national-coordination.md) | `services/governance-svc/tests/ci/test_county_coord.py` | 8 | Constitution Art.189, Elections Act s.57 |
| M57 | intl-observer-coord | governance | [docs/plans/modules/international-observer-coordination.md](../plans/modules/international-observer-coordination.md) | `services/governance-svc/tests/ci/test_intl_observer.py` | 7 | Elections Act s.34, s.35 |
| M58 | parliamentary-judiciary-reporting | governance | [docs/plans/modules/election-court-it-interface.md](../plans/modules/election-court-it-interface.md) | `services/governance-svc/tests/ci/test_reporting.py` | 8 | Elections Act s.87, Constitution Art.93 |
| M59 | oag-interface | governance | [docs/plans/modules/budget-expenditure.md](../plans/modules/budget-expenditure.md) | `services/governance-svc/tests/ci/test_oag.py` | 7 | Constitution Art.229, PAC Act |
| M60 | medical-first-response | training | [docs/plans/modules/poll-worker-training-lms.md](../plans/modules/poll-worker-training-lms.md) | `services/training-svc/tests/ci/test_medical.py` | 6 | Occupational Safety and Health Act |
| M61 | secrecy-assurance-testing | ai-platform | [docs/plans/modules/ai-monitoring-stack.md](../plans/modules/ai-monitoring-stack.md) | `services/ai-platform-svc/tests/ci/test_red_team.py` | 8 | Elections Act s.81, DPA 2019 |
| M62 | transmission-legal-audit | governance | [docs/plans/modules/governance-legal-audit.md](../plans/modules/governance-legal-audit.md) | `services/governance-svc/tests/ci/test_legal_audit.py` | 11 | Elections Act s.74, s.83 |
| M63 | ballot-serial-chain | rts | [docs/plans/modules/ballot-serial-chain.md](../plans/modules/ballot-serial-chain.md) | `services/rts-svc/tests/ci/test_serial.py` | 10 | Elections Act s.70, s.72 |
| M64 | voter-list-inspection | portal | [docs/plans/modules/cvr-continuous-voter-register.md](../plans/modules/cvr-continuous-voter-register.md) | `services/portal-svc/tests/ci/test_inspection.py` | 10 | Elections Act s.9, s.10 |
| M65 | diaspora-logistics | inventory | [docs/plans/modules/diaspora-ballot-physical-logistics.md](../plans/modules/diaspora-ballot-physical-logistics.md) | `services/inventory-svc/tests/ci/test_diaspora_logistics.py` | 9 | Elections Act s.55, Kenya Customs Act |
| M66 | financial-forensics | governance | [docs/plans/modules/election-financial-forensics.md](../plans/modules/election-financial-forensics.md) | `services/governance-svc/tests/ci/test_forensics.py` | 10 | Elections Act s.90, Statistics Act |
| M67 | party-symbol-registry | crms | [docs/plans/modules/crms.md](../plans/modules/crms.md) | `services/crms-svc/tests/ci/test_symbol.py` | 7 | Elections Act s.70, s.72 |
| M68 | referendum-explainer | portal | [docs/plans/modules/by-election-operations.md](../plans/modules/by-election-operations.md) | `services/portal-svc/tests/ci/test_explainer.py` | 8 | Elections Act s.42, WCAG 2.2 AAA |
| M69 | long-term-archive | support-admin | [docs/plans/modules/long-term-archive.md](../plans/modules/long-term-archive.md) | `services/support-admin-svc/tests/ci/test_archive.py` | 9 | KNADS Act, Elections Act s.87 |

## Notes

- EC rows: edge cases per module defined in `docs/plans/08-edge-case-matrix.md` and tracked in `docs/plans/TRACEABILITY.yaml`.
- Test file paths are canonical conventions; actual test files created per WP implementation wave.
- M53 (voter-id-card) requires explicit scope activation before production use.
- Module spec links point to `docs/plans/modules/` which is authoritative for module details. This index provides cross-references only.
