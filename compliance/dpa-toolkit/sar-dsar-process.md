# SAR / DSAR Process Flow

Subject Access Request (SAR) and Data Subject Access Request (DSAR) handling under Kenya Data Protection Act 2019 s.26-29.

---

## 1. Receipt and Identity Verification

**SLA start time:** The 30-calendar-day clock starts when the request is received, regardless of channel.

### 1.1 Channels for submission

| Channel | Handler | Acknowledgement SLA |
|---|---|---|
| Online portal ([SAR_PORTAL_URL]) | Automated receipt; routed to DPO queue | Automated acknowledgement within 1 hour |
| Email to [DPO_EMAIL] | DPO inbox monitor | Human acknowledgement within 1 working day |
| Letter to the Commission headquarters | Scanning and digitisation on receipt | Human acknowledgement within 1 working day |
| In person at the Commission county office | Officer logs the request and forwards to DPO | Human acknowledgement within 1 working day |

### 1.2 Identity verification

Before processing a SAR, verify the identity of the requestor:

1. For online requests: authenticate via the VPRS portal using the same credentials used for registration (or National ID + OTP to registered phone number).
2. For email or letter requests: request a copy of National ID or equivalent government-issued identity document. Do not retain the document beyond the verification step; destroy securely after verification is confirmed.
3. For requests on behalf of another person (e.g., next of kin, legal representative): require proof of authority (letter of authority, power of attorney, or court order) in addition to identity documents for both the requestor and the data subject.
4. If identity cannot be verified within 10 working days of the request, inform the requestor and suspend the clock. The clock resumes when verification is completed.

---

## 2. Legal Basis Check and DPA §40 Exemptions

After identity verification, assess whether any exemptions apply before compiling the response:

| Exemption | Data Category | Applicable Modules | Effect |
|---|---|---|---|
| DPA s.40(2)(b): legal obligation | Voter register (Tier A/B), polling records, ballot records | M01, M02, M03, M05, M06, M07, M35, M53 | Cannot erase; inform data subject of exemption |
| DPA s.40(2)(c): public interest | Election result records, audit trails | M10, M14, M22 | Cannot erase; inform data subject |
| DPA s.40(2)(d): legal claims | Grievance cases under active proceedings | M23 | Cannot erase during proceedings; inform data subject |
| DPA s.26(2): national security | Records relating to ongoing security investigation | M26, M28 | May partially restrict disclosure; document the restriction |
| No exemption | Ancillary records (M04 unprocessed applications, M23 closed unescalated grievances, M42, M43 rejected accreditations) | M04, M23, M36, M42, M43 | Full erasure or disclosure applies |

Document the exemption determination in the SAR case record with the specific statutory provision and reasoning.

---

## 3. Data Extraction Process

### 3.1 Per-service extraction

Each service exposes a SAR extract endpoint available only to the DPO service account. The endpoint returns all personal data held about a given subject, formatted as JSON.

| Service | SAR endpoint | Data returned |
|---|---|---|
| bvr-svc (M02, M35, M36, M53) | `GET /internal/sar/{subject_id}` | Voter record, registration events, card issuance |
| biometric-edge-svc (M01, M05) | `GET /internal/sar/{subject_id}` | Biometric template metadata (not raw template), registration events |
| abis-svc (M03) | `GET /internal/sar/{subject_id}` | Match request log (anonymised opponent IDs), adjudication outcomes |
| vprs-svc (M04) | `GET /internal/sar/{subject_id}` | Application status, submitted fields |
| bvi-svc (M06, M07) | `GET /internal/sar/{subject_id}` | Identification events, objections |
| ecc-svc (M23) | `GET /internal/sar/{subject_id}` | Grievance cases owned by subject |
| support-admin-svc (M25, M27, M33, M55) | `GET /internal/sar/{subject_id}` | Special voter events, attendance records, payroll (for staff only) |
| training-svc (M38, M42, M43) | `GET /internal/sar/{subject_id}` | Enrolment, certificates, accreditation status |

All SAR extract calls are logged as an audit event in the transparency log with: DPO operator ID, subject ID, timestamp, purpose = "SAR_RESPONSE".

### 3.2 Biometric data handling

Raw biometric templates are not disclosed in a SAR response. The response includes:
- Template ID (UUIDv7)
- Capture date
- Capture station ID
- Quality score tier (low/medium/high; not the numeric value)
- Whether a duplicate flag was raised and its resolution outcome

This satisfies the data subject's right to know that biometric data is held and how it was used, without creating a security risk by disclosing the template itself.

---

## 4. Response Compilation and SLA

**SLA:** 30 calendar days from receipt of a valid, verified request (DPA s.27(2)).

**Extension:** The SLA may be extended by a further 60 calendar days if the request is complex or numerous. The data subject must be informed of the extension and the reason within the first 30 days.

### 4.1 Response format

The response must include:

1. Confirmation that personal data is (or is not) held about the data subject.
2. A copy of all personal data held, in a structured, commonly used, machine-readable format (JSON or CSV preferred; PDF acceptable for in-person requests).
3. Purposes of processing for each category of data.
4. Categories of recipients.
5. Retention period for each category.
6. Any automated decision-making and the logic involved (reference DPA §41 register entries).
7. Information about the right to lodge a complaint with the ODPC ([odpc@odpc.go.ke](mailto:odpc@odpc.go.ke)).
8. For each category where disclosure is restricted or data cannot be erased, the specific statutory provision relied upon and the reason.

Deliver the response:
- Online requestors: via the SAR portal (password-protected download link, valid for 14 days)
- Email requestors: encrypted attachment (password communicated via a separate channel)
- In-person requestors: printed and sealed letter; or secure file transfer if agreed

---

## 5. Partial Exemption Handling

Where some data is subject to an exemption and some is not, provide the non-exempt data and redact the exempt data. Clearly explain to the data subject which categories are redacted and under which statutory provision.

Example template:

> Dear [SUBJECT_NAME],
>
> We are providing all personal data held about you that we are able to disclose. The following categories have been withheld under the Kenya Data Protection Act 2019 [SECTION] because [REASON]:
>
> - [Category 1]: [Statutory basis and reason]
>
> You have the right to object to this restriction by contacting the ODPC.

---

## 6. Appeal Process

If the data subject is dissatisfied with the response:

1. Internal review: the data subject may request a review by the DPO within 14 days of receiving the response. The DPO will respond within 15 working days.
2. ODPC complaint: the data subject may lodge a complaint directly with the ODPC at [odpc@odpc.go.ke](mailto:odpc@odpc.go.ke) or [www.odpc.go.ke](http://www.odpc.go.ke).
3. Tribunal: the data subject may appeal an ODPC decision to the Data Protection Tribunal under DPA s.76.

---

## 7. Record-Keeping

Retain the following for each SAR case for a minimum of 5 years:

| Record | Retention |
|---|---|
| SAR case log (request received, identity verified, response sent dates) | 5 years |
| Identity verification outcome (not the ID document itself) | 5 years |
| Copy of the response sent | 5 years |
| Exemption determination document | 5 years |
| Extension notices sent | 5 years |

SAR case logs are stored in the DPO case management system and are themselves subject to access restriction; only the DPO and designated deputies may access them.
