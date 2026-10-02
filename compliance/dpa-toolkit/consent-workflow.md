# Consent Workflow Templates

Consent applies as a lawful basis only where processing is genuinely voluntary and the data subject has a meaningful choice. For eel, consent is relevant primarily for:

- M04 (vprs): voluntary online voter registration channel
- M23 (grievance-channels): whistleblower anonymous reporting opt-in

For most other modules, the lawful basis is public task (Elections Act) and consent is not applicable or appropriate.

---

## 1. Consent Collection UI Flow

### 1.1 Pre-consent information (DPA s.30 notice)

Before collecting any personal data under consent, display all of the following in plain English and Swahili (and other applicable language per locale):

1. Identity of the data controller: the electoral authority (the Commission)
2. Contact details of the Data Protection Officer: [DPO_EMAIL]
3. Purpose of processing and legal basis: [PURPOSE], Consent under DPA s.30(1)(a)
4. Categories of personal data to be collected: [LIST_FIELDS]
5. Recipients or categories of recipients: [LIST_RECIPIENTS or "None - internal only"]
6. Retention period: [RETENTION_PERIOD]
7. Rights of the data subject: access, rectification, erasure, restriction, portability, objection, lodge complaint with ODPC
8. Whether provision of data is a statutory or contractual requirement, and consequences of not providing it
9. Existence of automated decision-making, including profiling, and meaningful information about the logic involved

This notice must be presented before the consent checkbox or button, not buried in a privacy policy link.

### 1.2 Age verification (voters)

Voters must be 18 years or older (Elections Act s.83). Before collecting any voter personal data via the VPRS channel:

1. Display the age requirement prominently.
2. Collect date of birth.
3. Calculate age server-side. If age is below 18, terminate the flow immediately. Do not store the attempted session beyond the current request.
4. For borderline cases (date of birth within 30 days of the 18th birthday), flag for manual verification by a returning officer before the application is accepted.

### 1.3 Consent capture

Use an explicit, unbundled consent mechanism:

- One checkbox per distinct processing purpose. Do not bundle multiple purposes into one checkbox.
- Checkboxes must be unchecked by default. Pre-ticked boxes are not valid consent under DPA s.30.
- Label each checkbox clearly: "I consent to [specific purpose] by the Commission."
- The submit button must only become active after all mandatory checkboxes are ticked.

### 1.4 Double opt-in for whistleblower reporting (M23)

When a data subject chooses the anonymous whistleblower channel:

1. Collect the contact detail (email or phone) for the confirmation step.
2. Send a one-time link or code to that contact detail.
3. Do not store the full submission until the data subject confirms via the link or code.
4. Confirmation link expires after 24 hours.
5. If not confirmed within 24 hours, delete all temporary data.
6. After confirmation, anonymise the contact detail used for verification (replace with a salted hash for deduplication only).

---

## 2. Consent Record Schema

Store one consent record per data subject per consent version. Schema (YAML representation of the database columns):

```yaml
consent_record:
  id: uuid7                     # UUIDv7 primary key
  subject_id: uuid7             # Reference to data subject; anonymised hash for whistleblower
  module_id: string             # e.g. "M04" or "M23"
  purpose_code: string          # e.g. "VPRS_REGISTRATION" or "WHISTLEBLOWER_REPORT"
  consent_version: string       # Version of the consent notice shown, e.g. "2026-01"
  given_at: timestamp_utc       # ISO 8601 timestamp when consent was given
  ip_address_hash: string       # SHA-256 hash of IP address (not stored in plaintext)
  user_agent_hash: string       # SHA-256 hash of user agent string
  channel: string               # "web" | "ussd" | "mobile"
  locale: string                # Locale of consent notice shown, e.g. "en-KE" or "sw-KE"
  withdrawn_at: timestamp_utc   # Null if not withdrawn; set on withdrawal
  withdrawal_reason: string     # Optional; provided by subject on withdrawal
  audit_trail_id: uuid7         # Reference to immutable audit event in transparency log
```

Do not store the raw IP address. Hash it to satisfy analytics needs without retaining a personal data point.

---

## 3. Withdrawal Flow

Data subjects have the right to withdraw consent at any time (DPA s.30(3)). Withdrawal must be as easy as giving consent.

### 3.1 Withdrawal channels

- Online: self-service portal (same channel as consent was given)
- In person: the Commission county office (with identity verification)
- Email: [CONSENT_WITHDRAWAL_EMAIL]
- Phone: [CONSENT_WITHDRAWAL_PHONE] (business hours)

### 3.2 Withdrawal processing steps

1. Receive withdrawal request; verify identity of the data subject.
2. Look up the consent record by subject identifier and purpose code.
3. Set `withdrawn_at` to the current timestamp and `withdrawal_reason` if provided.
4. Trigger downstream processing:
   a. For M04 (VPRS): if application is in-flight and not yet submitted to the register, halt processing and soft-delete the application.
   b. For M04 (VPRS): if the application has already been processed into the register, inform the data subject that the register record is subject to Elections Act s.32 statutory retention and cannot be erased by consent withdrawal; direct to the deregistration process (M36) if applicable.
   c. For M23 (whistleblower): anonymise any remaining identifying fields; retain only the anonymised grievance content if it has been escalated to the Commission.
5. Record the withdrawal in the audit trail.
6. Send a withdrawal confirmation to the data subject within 1 working day.
7. Do not use the withdrawn data for the purpose to which consent was given, from the withdrawal timestamp onwards.

### 3.3 Consequences of withdrawal

Inform the data subject, at the time of withdrawal, of any consequences:

- M04 VPRS: if registration has not yet been completed, the application will be cancelled. If already registered, withdrawal of consent does not remove the voter from the register (statutory retention applies; only deregistration under Elections Act s.15 can do that).
- M23 whistleblower: the grievance case will be closed unless it has been escalated, in which case it continues under the public task lawful basis.

---

## 4. Consent Notice Version Control

Each change to a consent notice must be assigned a new version identifier in the format `YYYY-MM` (year and month). When the notice changes materially:

1. Increment the version identifier.
2. For existing data subjects: re-obtain consent if the new purpose is materially different. If only wording changed, re-notice is sufficient without re-consent.
3. Log the version transition in the audit trail.
4. Keep prior consent notice text in `compliance/dpa-toolkit/consent-notices/` with the version as the filename, e.g. `M04-VPRS_REGISTRATION-2026-01.md`.
