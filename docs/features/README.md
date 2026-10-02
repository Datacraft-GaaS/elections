# Feature docs index

This directory contains the runtime walkthrough for each IEMS module: how it works in production, what aggregates it owns, how its API surface is shaped, and what the honest caveats are.

**Three complementary views of each module:**

- `docs/features/<slug>.md` (this directory) -- runtime walkthrough: aggregates, pipelines, API tables, integrity mechanisms, statute alignment, caveats.
- `docs/plans/modules/<slug>.md` -- canonical specification: requirements, work-package assignment, size estimate, upstream spec reference.
- `services/<svc>/docs/CAPABILITIES.md` -- service-level algorithm catalogue: cryptographic primitives, performance targets, integration contracts.

---

## Module index

| M-ID | Slug | Owning service | Purpose | Feature doc |
|---|---|---|---|---|
| M01 | bvr-device | biometric-edge-svc | Capture biographic and biometric data for citizens registering to vote, offline-first on KIEMS kit. | [bvr-device.md](bvr-device.md) |
| M02 | bvr-backend | bvr-svc | Central authoritative voter register: ingests enrolment records, maintains demographics and biometric links. | [bvr-backend.md](bvr-backend.md) |
| M03 | abis | abis-svc | Deduplicate the voter register at 30M+ scale using multi-modal biometric matching. | [abis.md](abis.md) |
| M04 | vprs | vprs-svc | Citizen pre-fill and identity verification for voter registration online. | [vprs.md](vprs.md) |
| M05 | bvi-device | biometric-edge-svc | Authenticate every voter presenting at a polling station on election day. | [bvi-device.md](bvi-device.md) |
| M06 | bvi-backend | bvi-svc | Central polling-day authority: distribute per-station register extracts, aggregate BVI events. | [bvi-backend.md](bvi-backend.md) |
| M07 | bvv | bvi-svc | Statutory 30-day public inspection of the voter register. | [bvv.md](bvv.md) |
| M08 | rts-device | rts-svc | Presiding Officer scans Form 34A at close of poll and transmits signed results. | [rts-device.md](rts-device.md) |
| M09 | rts-ro | rts-svc | Returning Officer tallying station receives and tallies signed Form 34A batches. | [rts-ro.md](rts-ro.md) |
| M10 | rts-backend | rts-svc | Central RTS backend: ingest signed form batches, verify signatures, aggregate results. | [rts-backend.md](rts-backend.md) |
| M11 | public-portal | portal-svc | Statutory public results portal: serve signed constituency and national results. | [public-portal.md](public-portal.md) |
| M12 | crms | crms-svc | Candidate and party Registration Management System. | [crms.md](crms.md) |
| M13 | public-verifier-app | field-mobile-svc | Multi-channel citizen app to verify participation in the election. | [public-verifier-app.md](public-verifier-app.md) |
| M14 | reproducibility-bundle | transparency-svc | Assemble and publish the statutory post-election reproducibility package. | [reproducibility-bundle.md](reproducibility-bundle.md) |
| M15 | iab-dashboard | ecc-svc | Independent Audit Board real-time dashboard with hardware-key-authenticated access. | [iab-dashboard.md](iab-dashboard.md) |
| M16 | election-command-centre | ecc-svc | National situational awareness loop for the Commission leadership on election day. | [election-command-centre.md](election-command-centre.md) |
| M17 | speech-integrity | ai-platform-svc | Real-time monitoring of public discourse for incitement and disinformation. | [speech-integrity.md](speech-integrity.md) |
| M18 | boundary-transparency | boundary-svc | Publish authoritative electoral boundary data: constituency, ward, polling centre. | [boundary-transparency.md](boundary-transparency.md) |
| M19 | queue-transparency | queue-svc | Real-time polling station queue monitoring. | [queue-transparency.md](queue-transparency.md) |
| M20 | inventory-management | inventory-svc | End-to-end asset lifecycle for election materials: kits, ballot papers, seals. | [inventory-management.md](inventory-management.md) |
| M21 | rla-audit-tool | governance-svc | Risk-Limiting Audit tool: post-election statistical sampling of paper ballots. | [rla-audit-tool.md](rla-audit-tool.md) |
| M22 | campaign-finance-transparency | transparency-svc | Statutory campaign finance disclosure: contributions and expenditure returns. | [campaign-finance-transparency.md](campaign-finance-transparency.md) |
| M23 | grievance-channels | ecc-svc | Multi-channel grievance intake for citizens, observers, and whistleblowers. | [grievance-channels.md](grievance-channels.md) |
| M24 | diaspora-voting | biometric-edge-svc | BVR and BVI workflows at embassy polling stations for Kenyan citizens abroad. | [diaspora-voting.md](diaspora-voting.md) |
| M25 | special-voter-workflows | support-admin-svc | Legally compliant workflows for five special voter categories. | [special-voter-workflows.md](special-voter-workflows.md) |
| M26 | anomaly-fraud-detection | ai-platform-svc | Seven-layer detection stack for rigging attempt detection in production streams. | [anomaly-fraud-detection.md](anomaly-fraud-detection.md) |
| M27 | intent-preservation | support-admin-svc | Minimise rejected and spoilt ballots through ballot optimisation and accessible design. | [intent-preservation.md](intent-preservation.md) |
| M28 | ai-monitoring-stack | ai-platform-svc | Governed ensemble of AI models around but never inside the result-determination pipeline. | [ai-monitoring-stack.md](ai-monitoring-stack.md) |
| M29 | mobile-citizen-verify | field-mobile-svc | Citizen mobile app for verifying participation and registration status. | [mobile-citizen-verify.md](mobile-citizen-verify.md) |
| M30 | mobile-party-agent | field-mobile-svc | Mobile tool for accredited party agents at polling stations. | [mobile-party-agent.md](mobile-party-agent.md) |
| M31 | mobile-poll-clerk | field-mobile-svc | Mobile app for Deputy Presiding Officers and polling clerks (~92,000 on polling day). | [mobile-poll-clerk.md](mobile-poll-clerk.md) |
| M32 | mobile-field-support | field-mobile-svc | Mobile console for field support engineers on election day. | [mobile-field-support.md](mobile-field-support.md) |
| M33 | mobile-personnel-dispatch | field-mobile-svc | Dispatch, attend, and track ~250,000 polling-day workers. | [mobile-personnel-dispatch.md](mobile-personnel-dispatch.md) |
| M34 | mobile-observer | field-mobile-svc | Structured observation and incident reporting for accredited observers. | [mobile-observer.md](mobile-observer.md) |
| M35 | cvr-continuous-voter-register | bvr-svc | Maintain the voter register as a living record between election cycles. | [continuous-voter-registration.md](continuous-voter-registration.md) |
| M36 | voter-deregistration-appeal | bvr-svc | Statutory workflow for voter deregistration and appeal adjudication. | [voter-deregistration-appeal.md](voter-deregistration-appeal.md) |
| M37 | voter-education-cms | training-svc | Manage, version, fact-check, translate, and distribute voter education content. | [voter-education-cms.md](voter-education-cms.md) |
| M38 | poll-worker-training-lms | training-svc | Deliver, assess, and certify training for ~250,000 poll workers. | [poll-worker-training-lms.md](poll-worker-training-lms.md) |
| M39 | ballot-printing-chain | inventory-svc | Track ballot papers from print authorisation through serial range gazettal to dispatch. | [ballot-printing-chain.md](ballot-printing-chain.md) |
| M40 | election-court-it-interface | governance-svc | Petition evidence submission to High Court and Supreme Court. | [election-court-it-interface.md](election-court-it-interface.md) |
| M41 | post-election-review | governance-svc | Structured lessons-learned from all stakeholder categories after each election. | [post-election-review.md](post-election-review.md) |
| M42 | media-accreditation-portal | training-svc | Accreditation of journalists, photographers, and broadcasters. | [media-accreditation-portal.md](media-accreditation-portal.md) |
| M43 | observer-accreditation-portal | training-svc | Accreditation of domestic and international election observers. | [observer-accreditation-portal.md](observer-accreditation-portal.md) |
| M44 | turnout-forecasting-analytics | ai-platform-svc | Pre-election turnout forecasting from CVR demographics and historical data. | [turnout-forecasting-analytics.md](turnout-forecasting-analytics.md) |
| M45 | party-registration-management | crms-svc | Automate the full lifecycle of political party registration and renewal. | [party-registration-management.md](party-registration-management.md) |
| M46 | party-primaries-oversight | crms-svc | Oversight and dispute resolution for party primaries. | [party-primaries-oversight.md](party-primaries-oversight.md) |
| M47 | nomination-fee-payment | crms-svc | Nomination fee schedules, M-Pesa C2B collection, waivers, and receipts. | [nomination-fee-payment.md](nomination-fee-payment.md) |
| M48 | recall | governance-svc | Constitutional recall petition process under Article 104. | [recall.md](recall.md) |
| M49 | referendum-mechanics | governance-svc | Configure and operate referendum elections under Articles 255-257. | [referendum-mechanics.md](referendum-mechanics.md) |
| M50 | by-election-operations | governance-svc | Full lifecycle of by-elections triggered by recall, vacancy, or nullification. | [by-election-operations.md](by-election-operations.md) |
| M51 | result-declaration-certification | rts-svc | Event-sourced declaration lifecycle producing HSM-countersigned certificates of election. | [result-declaration-certification.md](result-declaration-certification.md) |
| M52 | election-calendar-gazette | portal-svc | Publish and manage the statutory election calendar and gazette notices. | [election-calendar-gazette.md](election-calendar-gazette.md) |
| M53 | voter-id-card | bvr-svc | Voter ID card batch production, collection records, and blacklist propagation (<= 30 min SLA). | [voter-id-card.md](voter-id-card.md) |
| M54 | budget-expenditure | support-admin-svc | the Commission budget planning, procurement, payment vouchers, CoB reports, IFMIS + PPRA integration. | [budget-expenditure.md](budget-expenditure.md) |
| M55 | hr-payroll | support-admin-svc | HR and payroll for ~250k election staff: applications, contracts, M-Pesa disbursement, KRA/NSSF/NHIF returns. | [hr-payroll.md](hr-payroll.md) |
| M56 | county-national-coordination | governance-svc | MoUs, county dataset approvals, ministerial coordination, cross-agency incidents, SAML federation. | [county-national-coordination.md](county-national-coordination.md) |
| M57 | international-observer-coordination | governance-svc | International observer mission management, briefings, observation reports, joint statements. | [international-observer-coordination.md](international-observer-coordination.md) |
| M58 | parliamentary-judiciary-reporting | governance-svc | Statutory reports to Parliament, testimony management, judiciary handovers, SC pre-registration. | [parliamentary-judiciary-reporting.md](parliamentary-judiciary-reporting.md) |
| M59 | oag-interface | governance-svc | Scoped, time-boxed read-only OAG audit engagements with HSM-signed evidence packages. | [oag-interface.md](oag-interface.md) |
| M60 | medical-first-response | training-svc | Medical incident reporting, dispatch, and cluster alerts on election day. No PHI stored. | [medical-first-response.md](medical-first-response.md) |
| M61 | secrecy-assurance-testing | ai-platform-svc | Red-team engagements on isolated staging clones; IAB attestation required to close findings. | [secrecy-assurance-testing.md](secrecy-assurance-testing.md) |
| M62 | transmission-legal-basis-audit | governance-svc | 100% audit of RTS transmission events against a CEL statutory rules engine; rule-drift monitor. | [transmission-legal-basis-audit.md](transmission-legal-basis-audit.md) |
| M63 | ballot-serial-chain | rts-svc | Event-sourced ballot serial custody chain; result-hold flags block RTS until reconciliation is complete. | [ballot-serial-chain.md](ballot-serial-chain.md) |
| M64 | voter-list-public-inspection | portal-svc | Roll snapshots, objections, correction claims, determinations, and auto-referral packets. T-60 idempotent scheduler. | [voter-list-public-inspection.md](voter-list-public-inspection.md) |
| M65 | diaspora-ballot-physical-logistics | inventory-svc | Physical custody chain for diaspora ballot materials; MFA at every transfer; KRA + airline pre-clearance. | [diaspora-ballot-physical-logistics.md](diaspora-ballot-physical-logistics.md) |
| M66 | election-financial-forensics | governance-svc | Post-election RD/IV/DiD forensic analysis of campaign spending; FRC referral packets + replication packages. | [election-financial-forensics.md](election-financial-forensics.md) |
| M67 | party-symbol-colour-registry | crms-svc | Symbol pHash similarity detection and Delta-E colour distinctiveness enforcement; ballot design packages for printer. | [party-symbol-colour-registry.md](party-symbol-colour-registry.md) |
| M68 | referendum-explainer | portal-svc | Accessible referendum explainers: plain language, fact-check annotations, TTS audio, KSL video. WCAG 2.1 AAA. | [referendum-explainer.md](referendum-explainer.md) |
| M69 | long-term-archive | support-admin-svc | BagIt v1.0 archive collections, OAIS preservation, DataCite DOI minting, KNADS handover, destruction certificates. | [long-term-archive.md](long-term-archive.md) |

---

*Feature docs that are not yet written appear with a placeholder link. The canonical spec and design intent for every module lives in `docs/plans/modules/<slug>.md` regardless of whether the feature doc exists.*
