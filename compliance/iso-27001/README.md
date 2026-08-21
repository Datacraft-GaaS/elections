# ISO 27001:2022 ISMS Evidence

This directory contains the information security management system (ISMS) documentation for eel (IEBC Integrated Election Management System) aligned to ISO 27001:2022.

## Files

| File | Purpose |
|---|---|
| `isms-scope.md` | ISMS scope statement (Clause 4.3) |
| `annex-a-mapping.yaml` | All 93 Annex A controls mapped to platform artefacts and policies |
| `soa.yaml` | Statement of Applicability (SoA) per ISO 27001:2022 Clause 6.1.3(d) |
| `risk-register-template.yaml` | Risk register with 20 illustrative entries |
| `internal-audit-checklist.yaml` | Internal audit checklist per ISO 27001:2022 Clauses 4 through 10 |
| `management-review-agenda.md` | Quarterly management review meeting agenda template |

## Usage

1. Use `annex-a-mapping.yaml` and `soa.yaml` together: the SoA is the formal inclusion/exclusion declaration; the mapping is the evidence of implementation.
2. Populate the risk register with IEBC-specific risks before the first management review.
3. The internal audit checklist is completed by the internal audit team at least once per year.
4. Management reviews must be held at planned intervals (recommended: quarterly) using the agenda template.
5. All ISMS records must be retained for the duration of the certification plus 3 years.

## Certification target

ISO 27001:2022 certification covering the eel platform, all services, and the IEBC ICT environment as defined in `isms-scope.md`. Target certification body: [CERTIFICATION_BODY].
