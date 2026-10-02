# Data Protection Impact Assessment (DPIA)

Module: [MODULE_ID] [MODULE_NAME]
Version: 1.0
Date: [YYYY-MM-DD]
Author: [AUTHOR_NAME], [ROLE]
Status: DRAFT | UNDER_REVIEW | APPROVED

---

## 1. Purpose and Scope

**Module description:** [Describe what this module does, which data subjects it affects, and the processing activities it performs.]

**Processing activities in scope:**
- [Activity 1, e.g., biometric capture and storage]
- [Activity 2]
- [Activity 3]

**Data subjects:** [e.g., registered Kenyan voters, the Commission staff, accredited observers]

**Trigger for this DPIA:** [New module / significant change / periodic review]

**Legal obligation to conduct DPIA:** Kenya DPA 2019 §31: a DPIA is mandatory when processing is likely to result in high risk to the rights and freedoms of data subjects, particularly processing at scale involving special categories of data.

---

## 2. Data Controller

| Field | Value |
|---|---|
| Controller name | the electoral authority (the Commission) |
| Registered address | Anniversary Towers, University Way, Nairobi, Kenya |
| Contact email | [DPO_EMAIL] |
| Data Protection Officer | [DPO_NAME] |
| DPO registration number | [ODPC_REGISTRATION_NO] |
| Processor (if applicable) | [PROCESSOR_NAME or "None"] |
| Processing location | Primary DC: [DC_LOCATION]; Mirror DC: [MIRROR_DC_LOCATION] |

---

## 3. Personal Data Categories

| Field / Entity | Data Tier | Sensitivity | Lawful Basis (DPA §30) | Purpose |
|---|---|---|---|---|
| [Field 1, e.g., National ID Number] | A | High | Public task, Elections Act §32 | Voter identity verification |
| [Field 2, e.g., Biometric Template] | A | Very High | Public task, Elections Act §5(1)(b) | Biometric deduplication |
| [Field 3, e.g., Polling Station Assignment] | B | Medium | Public task, Elections Act §32 | Voter routing and turnout |
| [Field 4] | [Tier] | [Sensitivity] | [Basis] | [Purpose] |

**Special categories of data (DPA §26):** [List any data falling under special categories, e.g., biometric data. State how the DPA §27 condition is satisfied.]

**Data minimisation assessment:** [Confirm each field is necessary and sufficient. Any field that can be omitted or replaced with an identifier should be so treated.]

---

## 4. Necessity and Proportionality

**Necessity:** [Explain why the processing is necessary to achieve the stated purpose. Is there a less privacy-intrusive alternative?]

**Proportionality:** [Is the volume, sensitivity, and retention period proportionate to the public interest served? Reference Elections Act provisions.]

**Right to erasure (DPA §40):** [State whether the right to erasure applies. For Tier A/B data in the voter register, the legal-obligation exception under §40(2)(b) applies for the statutory retention period. For ancillary data such as grievance records, soft-delete followed by hard-purge applies.]

**Automated processing (DPA §41):** [State whether any decision with legal or significant effect is made solely by automated means. If yes, reference the DPA §41 register entry.]

---

## 5. Risk Assessment

| Risk Description | Likelihood (1-5) | Severity (1-5) | Score | Mitigation | Residual Risk |
|---|---|---|---|---|---|
| Unauthorised access to biometric templates | 2 | 5 | 10 | TDE + column-level encryption; SpiceDB RBAC; purpose-of-processing header enforced | Low (2) |
| Data subject enumeration via public API | 3 | 4 | 12 | Anti-enumeration rate limiting; no bulk export without IAB approval | Medium (4) |
| Insider threat: privileged admin exfiltration | 2 | 5 | 10 | SoD controls; audit events for every reveal(); SIEM alert on bulk access | Low (3) |
| Retention beyond statutory period | 2 | 3 | 6 | Automated nightly purge job with audit log; retention-schedule.yaml enforced in CI | Low (2) |
| Breach during transmission to/from kit | 3 | 4 | 12 | mTLS with hardware-attested kit certificate; HSM-signed payloads | Low (3) |
| [Additional risk] | [1-5] | [1-5] | [score] | [mitigation] | [level] |

**Overall risk rating before mitigation:** HIGH / MEDIUM / LOW
**Overall residual risk after mitigation:** HIGH / MEDIUM / LOW

---

## 6. Consultation

| Stakeholder | Consulted (Y/N) | Date | Summary of Feedback |
|---|---|---|---|
| Data Protection Officer | [Y/N] | [YYYY-MM-DD] | [Summary] |
| ODPC (if high risk) | [Y/N] | [YYYY-MM-DD] | [Summary or "N/A - residual risk not high"] |
| Data subjects (representative sample) | [Y/N] | [YYYY-MM-DD] | [Summary] |
| Legal Counsel | [Y/N] | [YYYY-MM-DD] | [Summary] |
| Security team | [Y/N] | [YYYY-MM-DD] | [Summary] |

---

## 7. DPO Sign-off

I confirm that this DPIA has been completed to the best of my knowledge, that the risks have been assessed accurately, and that the mitigations described are implemented or scheduled for implementation before the module goes live.

| | |
|---|---|
| DPO Name | [DPO_NAME] |
| Signature | [SIGNATURE] |
| Date | [YYYY-MM-DD] |
| ODPC reference (if consulted) | [REF or "N/A"] |

**Disagreement log:** [If the DPO disagrees with a decision in this DPIA, record the disagreement here per DPA §33(4).]

---

## 8. Review Schedule

| Trigger | Action |
|---|---|
| Prior to module go-live | DPO sign-off required |
| Annually or after each election cycle | Full review and re-approval |
| Significant change to processing | Partial or full re-assessment |
| Data breach affecting this module | Immediate review and update |
| DPA regulatory amendment | Review for compliance impact |

Next scheduled review: [YYYY-MM-DD]
