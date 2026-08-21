<title>Kenya DPA 2019 Toolkit</title>

# Kenya Data Protection Act 2019 Toolkit

Tools, templates, and process documentation for compliance with the Kenya Data Protection Act 2019
(Cap. 411C) and the Data Protection (General) Regulations 2021.

## Scope

The platform processes personal data of 22M+ registered voters including names, national ID
numbers, biometric templates (fingerprints and facial images), and residential location data.
All processing is conducted on behalf of the Commission as data controller. The system acts as
data processor.

## Directory contents

| File | Purpose |
|------|---------|
| `dpia-template.md` | Data Protection Impact Assessment template (Regulation 30) |
| `dpia-modules.yaml` | DPIA completion register: one entry per module processing personal data |
| `dpa-s41-automated-processing-register.yaml` | Register of automated processing activities (s.41) |
| `retention-schedule.yaml` | Data retention and erasure schedule per data category |
| `consent-workflow.md` | Voter consent capture and withdrawal process |
| `sar-dsar-process.md` | Subject Access Request and Data Subject Access Request handling |
| `odpc-breach-notification.md` | Office of the Data Protection Commissioner breach notification procedure |
| `README.md` | This file |

## Key obligations mapped

| DPA section | Obligation | Where addressed |
|-------------|-----------|----------------|
| s.25 | Lawful basis for processing | `dpia-modules.yaml` (lawful_basis field) |
| s.26 | Purpose limitation | `dpia-modules.yaml` (purpose field) |
| s.31 | Data minimisation | Architecture: biometric templates stored encrypted, not raw images |
| s.33 | Data subject rights | `sar-dsar-process.md` |
| s.41 | Automated decision-making | `dpa-s41-automated-processing-register.yaml` |
| s.43 | International transfers | ADR-0006 (data sovereignty: no cross-border transfer) |
| s.47 | Security measures | `compliance/iso-27001/controls-mapping.yaml` |
| s.53 | Data breach notification | `odpc-breach-notification.md` |
| Reg. 30 | DPIA for high-risk processing | `dpia-template.md`, `dpia-modules.yaml` |

## Running a DPIA for a new module

```bash
# Copy template and fill in module-specific fields
cp compliance/dpa-toolkit/dpia-template.md compliance/dpa-toolkit/dpia-<module>.md

# Register the DPIA in the module register
# Edit dpia-modules.yaml and add an entry with status: draft
```

DPIAs for biometric processing modules (abis-svc, bvr-svc, biometric-edge-svc) require sign-off
from the Commission's Data Protection Officer before gate G5.

## Handling a Subject Access Request

```bash
# Follow the step-by-step process in sar-dsar-process.md
# Generate a data export for a voter by national ID (preprod only)
just dpa-sar-export --national-id <id> --env preprod --output sar-export.zip
```

## Breach notification

In the event of a confirmed personal data breach:

1. Contain: isolate affected service (see degradation runbook)
2. Assess: determine categories and volume of data affected
3. Notify ODPC within 72 h using the template in `odpc-breach-notification.md`
4. Notify affected data subjects if high risk to rights and freedoms

## Limitations

- This toolkit covers the software platform's obligations only
- Physical security at polling stations is governed by separate operational procedures
- Legal interpretation of specific DPA provisions must be confirmed by qualified legal counsel

## Related docs

- ADR-0006 -- data sovereignty decisions
- `compliance/iso-27001/README.md` -- security controls (s.47 evidence)
- `compliance/elections-tech-regs/README.md` -- elections-specific data requirements
