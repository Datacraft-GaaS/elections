# GCC 15.4 IPR Acceptance Checklist

This checklist maps GCC 15.4 handover requirements to evidence paths and status. IEBC OIT signoff column is completed at handover.

## Legend

Status: DONE = evidence present and verified; PENDING = in progress; DEFERRED = planned for Gate G5.

| # | GCC 15.4 Requirement | Evidence path | Status | IEBC OIT signoff |
|---|---|---|---|---|
| 1 | Source code inventory covering all custom software | `compliance/ipr-handover/source-inventory-full.yaml` | DONE | |
| 2 | Copyright assignment executed for all custom artefacts | `compliance/ipr-handover/copyright-assignment.md` | DONE | |
| 3 | Software classification: Custom vs Standard | `compliance/ipr-handover/software-classification.yaml` | DONE | |
| 4 | Build instructions enabling independent rebuild | `compliance/ipr-handover/build-instructions.md` | DONE | |
| 5 | SBOM for all service images | `compliance/ipr-handover/sbom-index.md` | DONE | |
| 6 | Third-party licence report | `compliance/ipr-handover/licence-report.md` | DONE | |
| 7 | Reproducibility: bit-identical rebuild verified on x86_64 and aarch64 | `reproducibility/hash-manifests/` + gate G5 report | DEFERRED (Gate G5) | |
| 8 | Source escrow deposit manifest | `compliance/ipr-handover/gcc-15-4-handover-template.md` section 6 | PENDING | |
| 9 | SBOM attestations verifiable via Cosign/Sigstore | `compliance/ipr-handover/sbom-index.md` + `tools/supply-chain/` | DONE | |
| 10 | All 69 modules documented with acceptance tests | `docs/modules/README.md` + `docs/plans/TRACEABILITY.yaml` | DONE | |
| 11 | ASVS L3 compliance for election-critical modules | `compliance/iso-27001/` + `tools/asvs-check/` | PENDING (Gate G5 audit) | |
| 12 | WCAG 2.2 AAA compliance for public-facing portal | `compliance/elections-tech-regs/` + `tools/wcag-audit/` | PENDING (Gate G5 audit) | |
| 13 | DPIA completed for all PII-handling modules | `compliance/dpa-toolkit/` | PENDING | |
| 14 | ISO 27001 Annex A mapping | `compliance/iso-27001/annex-a-mapping.md` | PENDING | |
| 15 | Elections (Technology) Regs 2017 compliance harness | `compliance/elections-tech-regs/` | PENDING | |
| 16 | DR failover within SLO demonstrated in staging | `chaos/` gate reports | DEFERRED (Gate G5) | |
| 17 | All 19 runbooks exercised | `docs/runbooks/` + runbook exercise logs | PENDING | |
| 18 | Git commit history for Waves 0-5 | `compliance/ipr-handover/provenance/` | DONE | |
| 19 | Wave gate reports linked | `compliance/ipr-handover/provenance/wave-*.yaml` | DONE | |
| 20 | Handover runbook for physical transfer to IEBC OIT | `compliance/ipr-handover/handover-runbook.md` | DONE | |
| 21 | Physical media tarball procedure documented | `compliance/ipr-handover/handover-runbook.md` | DONE | |
| 22 | Key ceremony for HSM handover documented | `docs/runbooks/hsm-key-ceremony.md` | DONE | |
| 23 | No IEBC identifiers in code or package names | Memory directive + CI lint check | DONE | |
| 24 | Source inventory script provided for regeneration | `compliance/ipr-handover/scripts/generate-source-inventory.sh` | DONE | |

## Notes

- Items marked DEFERRED are gated on the Gate G5 completion criteria in `docs/plans/06-wave-5-governance.md`.
- IEBC OIT signoff column is left blank pending the physical handover ceremony described in `compliance/ipr-handover/handover-runbook.md`.
- This checklist must be reviewed by the IEBC OIT team before the handover ceremony.
