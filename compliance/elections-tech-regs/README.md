<title>Elections Technology Regulations Mapping</title>

# Elections (Technology) Regulations Mapping

Requirement-by-requirement mapping of the Elections (Technology) Regulations 2017 (LN 45/2017,
hereafter "ETR") to system components. Includes an automated test harness that verifies
regulatory obligations are met before each gate.

## Regulatory scope

The ETR governs:
- Registration of voters using biometric technology (Regs 3-8)
- Identification of voters at polling stations (Regs 9-11)
- Transmission of election results (Regs 12-16)
- Security and audit requirements (Regs 17-21)
- Technology certification requirements (Regs 22-24)

## Mapping approach

Each ETR regulation is mapped to:

1. One or more services/modules in the system
2. The functional requirements (FR codes) that implement it
3. The test cases (`@trace` annotations) that verify it
4. The evidence artefact produced at gate G5/G6

The mapping is maintained in `system-register.yaml` (service inventory per regulation) and
`regs-mapping.yaml` (regulation to FR/test trace).

## Regulation summary

| Regulation | Subject | Services | Evidence |
|-----------|---------|---------|---------|
| Reg. 3 | Biometric capture equipment certification | biometric-edge-svc, kit | Device certification docs |
| Reg. 4 | Voter registration database security | bvr-svc | ISO 27001 control 8.3, DPA DPIA |
| Reg. 5 | Deduplication of biometric records | abis-svc | ABIS accuracy report (FMR/FNMR) |
| Reg. 6 | Voter register audit trail | bvr-svc, transparency-svc | Transparency log entries |
| Reg. 7 | Register update procedures | bvr-svc | Change management process |
| Reg. 8 | Register verification before election | bvr-svc, bvi-svc | Pre-election verification report |
| Reg. 9 | Voter identification at polling station | biometric-edge-svc, bvi-svc | Kit acceptance tests |
| Reg. 10 | Fallback identification procedure | biometric-edge-svc (offline mode) | Degradation runbook |
| Reg. 11 | Prevention of double voting | bvi-svc, rts-svc (dedup fence) | Idempotency test suite |
| Reg. 12 | Results transmission system | rts-svc | Load test + chaos scenario 20 |
| Reg. 13 | Form 34A/34B digital signing | rts-svc, transparency-svc (HSM) | Signed form samples |
| Reg. 14 | Results transmission audit trail | transparency-svc | Merkle root per tally run |
| Reg. 15 | Public display of results | portal-svc | Live tally smoke test |
| Reg. 16 | Results reconciliation | ecc-svc | Reconciliation report in rehearsal |
| Reg. 17 | System security requirements | All services | ISO 27001 SoA, ASVS report |
| Reg. 18 | Access control to technology systems | Keycloak, SpiceDB | Authz golden tests |
| Reg. 19 | Audit logging | transparency-svc, Loki | Audit log retention proof |
| Reg. 20 | System testing before election | `verification/` harnesses | G6 evidence package |
| Reg. 21 | Independent security audit | `verification/security/` | Pen-test report |
| Reg. 22 | Technology certification | All tier-A services | Certification authority letter |
| Reg. 23 | Source code escrow | `compliance/ipr-handover/` | Escrow receipt |
| Reg. 24 | Post-election system preservation | Velero backup, MinIO archive | Backup verification report |

## Running the automated harness

```bash
# Validate mapping completeness (dry-mode)
just etr-check dry

# Run all testable regulation checks against preprod
just etr-check live ENV=preprod
```

The harness checks:
- All ETR regulations have at least one mapped test trace
- Cited test cases pass in CI
- Form 34A/34B signing chain verifiable end-to-end
- Transparency log Merkle roots match after rehearsal

## Expected artefacts

Written to `verification/reports/elections-tech-regs-<date>/`:

- `regs-compliance-matrix.json` -- per-regulation pass/fail
- `regs-compliance-matrix.html` -- colour-coded HTML for auditors
- `form-34-signing-proof.json` -- verified signing chain for sample forms
- `gate-g5-etr.sig` -- signed evidence for G5

## Limitations

- Regulation 22 certification requires a letter from an accredited certification authority:
  this harness verifies technical readiness only
- Regulation 23 escrow deposit is a physical/legal act; this harness produces the source archive
  but not the deposit receipt

## Related docs

- `compliance/ipr-handover/README.md` -- source code escrow
- `compliance/iso-27001/README.md` -- security requirements (Reg. 17)
- `verification/security/README.md` -- independent security audit evidence
- `services/rts-svc/docs/README.md` -- results transmission architecture
