<title>Security Verification Harness</title>

# Security Verification Harness

OWASP ASVS Level 3 verification plus automated pen-test tooling for the Kenya elections platform.
Wave-6 WP-6.5. Gate G6 requires a clean ASVS report and a signed pen-test summary.

## Scope

- All 18 services and their APIs (OpenAPI contracts in `packages/openapi-contracts/`)
- SPIRE/SVID mTLS configuration
- Keycloak OIDC flows
- SpiceDB authz schema (`.zed` files)
- OpenBao secret management
- Kubernetes RBAC and Kyverno policies
- Transparency log integrity

## ASVS Level 3 checks

ASVS v4.0.3 requirements mapped per service in `compliance/iso-27001/`. This harness automates
the testable subset:

```bash
# Run all ASVS automated checks (dry-mode: no destructive tests)
just security-verify dry

# Run including active pen-test modules (live-mode, preprod only)
just security-verify live ENV=preprod
```

## Tools used

| Tool | Purpose |
|------|---------|
| `nuclei` | CVE and misconfiguration scanning against all service endpoints |
| `trivy` | Container image CVE scanning |
| `checkov` | IaC policy checks (Kustomize, Terraform) |
| `semgrep` | SAST on all Rust, Python, Elixir, Dart sources |
| `gitleaks` | Secret scanning across repo history |
| `zap` | OWASP ZAP DAST scan against portal-svc and ecc-svc |
| `kube-bench` | CIS Kubernetes benchmark |
| `conftest` | OPA policy tests on Kubernetes manifests |
| `jwt-tool` | JWT algorithm confusion and claim injection tests |
| `spifee-verify` | SVID chain and trust domain validation |

## Modes

### dry-mode

Runs static analysis (semgrep, checkov, trivy, gitleaks, conftest) against local sources and
container images. No network connection to live services.

```bash
just security-verify dry
```

### live-mode

Adds DAST (ZAP, nuclei, jwt-tool) against pre-prod services. Requires pen-test change request.

```bash
just security-verify live ENV=preprod TARGETS=portal,ecc,bvr
```

## Expected artefacts

Written to `verification/reports/security-<date>/`:

- `asvs-results.json` -- per-requirement pass/fail with evidence links
- `nuclei-report.html` -- nuclei scan findings
- `trivy-report.json` -- image CVE report
- `semgrep-report.sarif` -- SAST findings (SARIF format, imported to GitHub Security tab)
- `zap-report.html` -- DAST findings
- `kube-bench-report.json` -- CIS benchmark results
- `pen-test-summary.md` -- narrative summary for G6 evidence package
- `gate-g6-security.sig` -- signed evidence for G6

## Pass criteria

- Zero ASVS Level 3 requirements with status "FAIL"
- No Critical or High CVEs in container images without a documented exception
- No secrets detected by gitleaks in repo history
- kube-bench: all Level 1 checks pass; Level 2 exceptions documented
- ZAP: no High-severity findings; Medium findings have remediation tickets

## Limitations

- Hardware HSM PKCS#11 tests require physical HSM access (not automated)
- Biometric data tests use synthetic probes only
- Red-team exercises (physical, social engineering) are out of scope for this harness

## Related docs

- `compliance/iso-27001/README.md` -- ASVS-to-ISO mapping
- `compliance/dpa-toolkit/README.md` -- data protection checks
- `packages/openapi-contracts/` -- API specs under test
- `docs/THREAT-MODEL.md` (per-service threat models)
