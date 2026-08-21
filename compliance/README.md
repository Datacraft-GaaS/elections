<title>Compliance Programme</title>

# Compliance Programme

Evidence, mappings, and toolkits for four compliance obligations that apply to the Kenya elections
platform (tender IEBC/OIT/01/2026-2027):

1. Kenya Data Protection Act 2019 (DPA) and the Data Protection (General) Regulations 2021
2. ISO 27001:2022 information security management
3. Elections (Technology) Regulations 2017 (LN 45/2017)
4. Intellectual property and source-code handover per GCC clause 15.4

## Wave context

WP-5.13 (compliance) produces initial evidence packages; WP-6.7 (completeness) references them.
Gate G5 requires signed compliance readiness statements. Gate G6 requires final signed evidence.

## Directory layout

```
compliance/
  dpa-toolkit/           Kenya DPA 2019 toolkit: DPIA templates, DSAR process, breach process
  iso-27001/             ISO 27001:2022 controls mapping and ASVS evidence links
  elections-tech-regs/   Elections (Technology) Regulations 2017 mapping and test harness
  ipr-handover/          Source handover process, IPR schedule, escrow instructions
  README.md              This file
```

## Compliance ownership

| Obligation | Owner | Evidence location |
|------------|-------|------------------|
| DPA 2019 | Data Protection Officer | `compliance/dpa-toolkit/` |
| ISO 27001:2022 | Information Security Manager | `compliance/iso-27001/` |
| Elections Tech Regs | Elections Technology Director | `compliance/elections-tech-regs/` |
| IPR handover | Project Director | `compliance/ipr-handover/` |

## Generating a compliance status report

```bash
just compliance-status   # prints RAG status for each obligation
just compliance-package  # bundles evidence into compliance/reports/g5-compliance.zip
```

## Dependencies

- `compliance/iso-27001/controls-mapping.yaml` up to date with current service inventory
- DPIA sign-offs in `compliance/dpa-toolkit/dpia-modules.yaml` for all modules processing
  personal data
- All 18 services listed in `elections-tech-regs/system-register.yaml`

## Related docs

- ADR-0006 -- data sovereignty and personal data handling
- ADR-0010 -- audit log and evidence retention
- `docs/audit/gate-G5.md` -- G5 evidence package
- `verification/security/README.md` -- ASVS evidence referenced by ISO 27001
