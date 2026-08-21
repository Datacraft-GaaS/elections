# ODPC Breach Notification Template and Drill Procedure

Kenya Data Protection Act 2019 s.43 requires notification to the ODPC within 72 hours of becoming aware of a personal data breach likely to result in risk to the rights and freedoms of data subjects.

---

## 1. Breach Categorisation

Classify the breach on two axes before notifying:

### 1.1 Type

| Type | Description | Example |
|---|---|---|
| Confidentiality breach | Unauthorised or accidental disclosure or access | Voter PII exfiltrated by external attacker |
| Integrity breach | Unauthorised or accidental alteration | Voter register record tampered without audit trail |
| Availability breach | Unauthorised or accidental destruction or loss of access | Database unavailable due to ransomware |

A breach may be more than one type simultaneously.

### 1.2 Severity

| Severity | Description | Notification requirement |
|---|---|---|
| Critical | Tier A data (biometric, ID number, biographical) breached at scale (more than 100 records) | Notify ODPC within 72 hours; notify affected data subjects without undue delay |
| High | Tier A data breached at small scale (fewer than 100 records) OR Tier B data breached at scale | Notify ODPC within 72 hours; assess whether to notify data subjects |
| Medium | Tier B or Tier D data breached; Tier A data involved but no likely real-world harm | Notify ODPC within 72 hours if risk to rights and freedoms; document internally |
| Low | Tier E ephemeral data; no personal data involved | No ODPC notification required; document internally |

---

## 2. Internal Escalation Flow

Upon discovery or suspicion of a breach, follow this escalation sequence:

1. Discoverer (any staff member or system alert) notifies the on-call security engineer immediately via the incident response channel (operations/helpdesk/).
2. On-call security engineer performs initial triage within 1 hour: confirm whether a breach has occurred, estimate scope, classify severity.
3. On-call security engineer escalates to the Data Protection Officer within 2 hours of confirmation.
4. DPO notifies the IEBC Director-General and Legal Counsel within 3 hours.
5. DPO determines, with Legal Counsel, whether the 72-hour ODPC notification threshold is met.
6. If yes: DPO initiates notification to ODPC. Target: notification sent within 48 hours of breach discovery to allow for any supplementary information before the 72-hour deadline.
7. DPO initiates notification to affected data subjects if required.
8. All breach-related communications are logged in the incident management system (operations/helpdesk/).

---

## 3. ODPC Notification Template

Send via email to [odpc@odpc.go.ke](mailto:odpc@odpc.go.ke) with subject: "Personal Data Breach Notification - [IEBC_BREACH_REF] - [DATE]"

---

**PERSONAL DATA BREACH NOTIFICATION**

**To:** Office of the Data Protection Commissioner (ODPC)
**From:** Independent Electoral and Boundaries Commission (IEBC)
**Data Protection Officer:** [DPO_NAME], [DPO_EMAIL]
**ODPC Registration Number:** [ODPC_REGISTRATION_NO]
**IEBC Breach Reference:** [BREACH_REF, e.g., BREACH-2026-001]
**Date of notification:** [YYYY-MM-DD HH:MM UTC]
**Date and time breach was discovered:** [YYYY-MM-DD HH:MM UTC]
**Date and time breach is believed to have commenced:** [YYYY-MM-DD HH:MM UTC or "Unknown - under investigation"]

**1. Nature of the breach**

[Describe in plain language what happened. Include: how the breach was discovered; what systems or data are affected; type of breach (confidentiality/integrity/availability).]

**2. Categories and approximate number of data subjects affected**

| Category | Approximate number |
|---|---|
| Registered voters | [NUMBER or "Under investigation"] |
| IEBC staff | [NUMBER or "N/A"] |
| Accredited media/observers | [NUMBER or "N/A"] |
| Other: [DESCRIBE] | [NUMBER or "N/A"] |

**3. Categories and approximate number of personal data records affected**

| Data category | Tier | Approximate records |
|---|---|---|
| [e.g., National ID numbers] | A | [NUMBER] |
| [e.g., Polling station assignments] | B | [NUMBER] |

**4. Likely consequences of the breach**

[Describe the likely real-world consequences for affected data subjects: identity theft risk, disenfranchisement risk, safety risk (especially for whistleblowers), reputational harm, etc.]

**5. Measures taken or proposed to address the breach and mitigate its effects**

[Describe immediate containment measures already taken. Describe planned remediation measures and timelines.]

**6. Contact for further information**

Name: [DPO_NAME]
Role: Data Protection Officer, IEBC
Email: [DPO_EMAIL]
Phone: [DPO_PHONE]
Available: [AVAILABILITY_HOURS]

**7. Further information to follow**

[If full information is not yet available, state: "This notification is made within the 72-hour statutory period. A supplementary notification with [LIST_MISSING_INFORMATION] will follow by [DATE]."]

---

## 4. Data Subject Notification Template

Use this template for notifying affected data subjects. Send via the most effective channel available (SMS to registered phone number, email, or publication on the IEBC website for large-scale breaches where individual contact is not feasible).

---

**IMPORTANT NOTICE: Your Personal Data May Have Been Affected**

From: Independent Electoral and Boundaries Commission (IEBC)
Date: [YYYY-MM-DD]
Reference: [BREACH_REF]

We are writing to inform you that IEBC has experienced a personal data security incident that may have affected your personal information.

**What happened**

[Brief plain-language description of the breach.]

**What information may have been affected**

[List the categories of personal data that may have been affected. Be specific but avoid jargon.]

**What we are doing**

[Describe the steps IEBC is taking to contain the breach, investigate it, and prevent recurrence.]

**What you can do**

- Be alert to any suspicious communications claiming to be from IEBC.
- If you receive any unexpected requests for your personal information, do not provide it without verifying the request directly with IEBC using contact details on our official website [iebc.or.ke].
- [Any other specific steps the data subject can take to protect themselves.]

**Contact us**

If you have questions about this incident, contact the IEBC Data Protection Office:
Email: [DPO_EMAIL]
Phone: [DPO_PHONE]
Hours: [HOURS]

You also have the right to lodge a complaint with the ODPC at [odpc@odpc.go.ke](mailto:odpc@odpc.go.ke).

---

## 5. Breach Drill Procedure

IEBC must conduct at least one tabletop breach notification drill per year, before each election cycle where a General Election is anticipated.

### 5.1 Drill schedule

| Activity | Timing | Owner |
|---|---|---|
| Schedule drill date | At least 6 weeks before drill | DPO |
| Identify drill participants | 4 weeks before drill | DPO |
| Prepare drill scenario | 2 weeks before drill | DPO + Security Team |
| Conduct drill | Drill date | DPO |
| Post-drill review document | Within 5 working days of drill | DPO |
| Update procedures based on findings | Within 30 days of drill | DPO |

### 5.2 Drill participants

- Data Protection Officer (chair)
- Director-General or designate
- Head of ICT
- Legal Counsel
- On-call security engineer
- Communications officer (for data subject notification element)
- Representative from affected service team (rotated per year)

### 5.3 Drill steps

1. Facilitator presents the breach scenario (no advance warning to participants beyond the date and duration).
2. Participants work through the internal escalation flow (section 2) using the scenario.
3. DPO drafts the ODPC notification letter (section 3) within a simulated 2-hour window.
4. Participants identify any gaps or uncertainties in the process.
5. Facilitator introduces a complication (e.g., DPO unavailable, ODPC portal down) and participants adapt.
6. Debrief: facilitator leads a 30-minute post-drill review.

### 5.4 Post-drill review document

The DPO produces a drill review document within 5 working days including:
- Drill date and participants
- Scenario used
- Timeline of decisions made during the drill
- Gaps identified
- Actions to address gaps (owner and due date for each)
- Whether a follow-up drill is needed before the election cycle

Store drill review documents in `compliance/dpa-toolkit/drill-records/YYYY-MM-DD-drill-review.md`.
