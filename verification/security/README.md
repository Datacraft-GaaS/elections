# Security Harness (WP-6.5)

Verification harness for the eel security posture. Scripts are designed to run in
pre-prod. Set `EEL_DR_MODE=dry` for syntax-only / CI dry runs.

## Coverage areas

| Area | Tools | Gate |
|---|---|---|
| SAST | Semgrep (with eel-specific rules) | zero critical/high |
| SCA | cargo-audit, cargo-deny, pip-audit, pnpm-audit | zero critical/high |
| Container | Trivy, Grype, Syft | zero critical/high |
| DAST | OWASP ZAP baseline, Nuclei templates | no critical findings |
| Fuzz | cargo-fuzz, 24 h schedule | zero new crashes |
| Pen-test | Scope + findings tracker | all open items tracked |
| ASVS L3 | tools/asvs-check --strict | zero L3 gaps |
| PQ-hybrid TLS | testssl.sh | TLS 1.3 + ML-KEM + X25519 negotiated |

## Quick start

```sh
# Dry run (syntax check + YAML validation, no network):
EEL_DR_MODE=dry bash sast-sca-container.sh
EEL_DR_MODE=dry bash dast/zap-baseline.sh
EEL_DR_MODE=dry bash fuzz/run-corpus.sh
EEL_DR_MODE=dry bash asvs-evidence-collector.sh
EEL_DR_MODE=dry bash pq-hybrid/testssl.sh
```

## References

- ADR-0005: TLS 1.3 + PQ hybrid (ML-KEM + X25519)
- ADR-0006: Supply chain (cargo-audit, cargo-deny, Trivy, Grype, Semgrep)
- `tools/asvs-check/` for ASVS L3 evidence collection
- `docs/runbooks/` for incident response escalation
