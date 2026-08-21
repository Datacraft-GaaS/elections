# Security Verification Report

**Report date**: 2026-08-21
**Ticket**: SEC-20260821
**Environment**: pre-prod (synthetic data, no PII)
**Operator(s)**: Alice K., Bob M.

## SAST / SCA / Container matrix

| Tool | Status | Critical | High | Notes |
|---|---|---|---|---|
| cargo-audit | run | 0 | 0 | pass |
| cargo-deny | run | 0 | 0 | pass |
| semgrep | run | 0 | 2 | 2 low-severity findings, reviewed, accepted |
| trivy | run | 0 | 0 | pass |
| grype | run | 0 | 0 | pass |
| bandit | run | 0 | 0 | pass |
| sobelow | skipped | 0 | 0 | Elixir services not in scope this sprint |

**Gate result**: PASS (zero critical/high)

## DAST findings

### ZAP baseline

- Target: https://nairobi.edge.eel.internal
- High alerts: 0
- Report: /tmp/eel-zap-report/zap-baseline-report.json

### Nuclei

- Templates: cves, exposures, misconfigurations, workflows/eel-*
- Findings (medium+): 0
- Custom workflow results: timing probe: no side channel detected; idempotency: correct

## Fuzz coverage delta and crashes

| Target | Iterations | Coverage delta | New crashes |
|---|---|---|---|
| envelope | 5 000 000 | +2.1% | 0 |
| canonical_json_rfc8785 | 5 000 000 | +1.4% | 0 |
| ed25519_verify | 10 000 000 | +0.8% | 0 |
| merkle_proof_decode | 5 000 000 | +3.2% | 0 |

**Total new crashes**: 0

## Pen-test open items

| ID | Title | Severity | Status |
|---|---|---|---|
| none | | | |

## ASVS L3 gaps

- Services checked: 19
- L3 gaps: 0
- Evidence file: verification/security/asvs-evidence.json

**Gate result**: PASS

## PQ negotiation result

| Edge URL | TLS 1.3 | Hybrid KEM | Classical fallback |
|---|---|---|---|
| nairobi.edge.eel.internal | yes | ML-KEM-768+X25519 | accepted |
| mombasa.edge.eel.internal | yes | ML-KEM-768+X25519 | accepted |
| kisumu.edge.eel.internal | yes | ML-KEM-768+X25519 | accepted |
| nakuru.edge.eel.internal | yes | ML-KEM-768+X25519 | accepted |
| eldoret.edge.eel.internal | yes | ML-KEM-768+X25519 | accepted |

## Sign-off

| Role | Name | Signature |
|---|---|---|
| Security officer | Alice K. | |
| Ops lead | Bob M. | |
| External pen-test lead | External Team Ltd | |

*Rendered from `verification/security/report-template.md.tmpl` with dry-run data.*
