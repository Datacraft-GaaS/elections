# DPA Toolkit

Compliance templates for Kenya Data Protection Act 2019 obligations applicable to the eel (IEBC Integrated Election Management System).

## Files

| File | Purpose |
|---|---|
| `dpia-template.md` | Master DPIA template; copy and fill in per module |
| `dpia-modules.yaml` | One entry per PII module (18 Tier A/B modules) with metadata |
| `retention-schedule.yaml` | Per-module retention rules, purge triggers, statutory refs |
| `consent-workflow.md` | Consent collection, withdrawal, and record templates |
| `sar-dsar-process.md` | Subject access request process flow and response templates |
| `odpc-breach-notification.md` | 72-hour ODPC breach notification template and drill procedure |
| `dpa-s41-automated-processing-register.yaml` | DPA §41 automated-decision register |

## Usage

1. For each Tier A or Tier B module, copy `dpia-template.md` and complete every `[PLACEHOLDER]`.
2. Update `dpia-modules.yaml` when a module moves from `template` to `approved`.
3. Run the annual ODPC breach drill using `odpc-breach-notification.md` section 5.
4. The Data Protection Officer (DPO) must sign each completed DPIA before the module goes to pre-production.
5. Review this toolkit after every election cycle or when DPA regulations are amended.

## Contacts

| Role | Placeholder |
|---|---|
| Data Protection Officer | [DPO_NAME], [DPO_EMAIL] |
| ODPC Liaison | [ODPC_OFFICER], odpc@odpc.go.ke |
| Legal Counsel | [LEGAL_COUNSEL_NAME] |
