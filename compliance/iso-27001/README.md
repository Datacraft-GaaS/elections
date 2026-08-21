<title>ISO 27001:2022 Controls Mapping</title>

# ISO 27001:2022 Controls Mapping

Evidence mapping for ISO/IEC 27001:2022 Annex A controls as they apply to the Kenya elections
platform. Supports the Commission's ISMS certification scope and feeds into the Wave-5 compliance
package (WP-5.13) and the Gate G5/G6 evidence bundles.

## ISMS scope statement

The ISMS covers the design, development, operation, and maintenance of the integrated elections
management system including all 18 production services, three-site Kubernetes infrastructure,
HSM-based signing, biometric processing pipeline, and associated development toolchain.

## Controls coverage summary

ISO 27001:2022 contains 93 controls across 4 themes and 11 clauses. The mapping file
`controls-mapping.yaml` records: control ID, title, applicability (Yes/No/Partial), implementation
evidence (file paths, ADR references, CI artefact URLs), and responsible owner.

| Theme | Controls | Applicable | Evidence location |
|-------|---------|-----------|------------------|
| Organisational (5.1-5.37) | 37 | 35 | `controls-mapping.yaml` |
| People (6.1-6.8) | 8 | 8 | HR process docs (separate) |
| Physical (7.1-7.14) | 14 | 10 | Data centre contracts (separate) |
| Technological (8.1-8.34) | 34 | 34 | `controls-mapping.yaml` + ASVS evidence |

## Key technological controls

| Control | Title | Evidence |
|---------|-------|---------|
| 8.2 | Privileged access rights | Keycloak roles, SpiceDB .zed schemas |
| 8.3 | Information access restriction | OIDC scopes, authz golden tests |
| 8.5 | Secure authentication | mTLS (SPIRE SVIDs), OIDC PKCE, MFA enforced |
| 8.7 | Protection against malware | Trivy CI scan, Falco runtime alerts |
| 8.8 | Management of technical vulnerabilities | Trivy + semgrep in CI, 30-day patching SLA |
| 8.9 | Configuration management | GitOps (ArgoCD), Kyverno admission control |
| 8.10 | Information deletion | DPA retention schedule, automated erasure jobs |
| 8.12 | Data leakage prevention | Network policies (Cilium), secrets in OpenBao |
| 8.15 | Logging | Loki + Tempo, audit log to transparency-svc |
| 8.16 | Monitoring activities | Prometheus + Grafana, Alertmanager |
| 8.20 | Networks security | Cilium CNI, zero-trust mTLS, envoy-gateway |
| 8.23 | Web filtering | WAF on ingress, rate limiting |
| 8.24 | Use of cryptography | ADR-0007 (crypto policy): Ed25519, AES-256-GCM |
| 8.25 | Secure development lifecycle | ADR-0001, spectral linting, nextest CI |
| 8.26 | Application security requirements | ASVS Level 3, OpenAPI contracts |
| 8.28 | Secure coding | Rust memory safety, semgrep SAST, clippy |
| 8.29 | Security testing in dev and acceptance | `verification/security/README.md` |
| 8.31 | Separation of dev, test, production | Three environments: local, staging, preprod/prod |
| 8.32 | Change management | ADR review process, ArgoCD sync gates |
| 8.33 | Test information | Synthetic fixtures only in non-prod |
| 8.34 | Protection of information systems during audit testing | Chaos Mesh scoped to chaos-testing namespace |

## Viewing the full mapping

```bash
# Pretty-print the controls mapping
just iso-controls-status

# Export as CSV for auditor review
just iso-controls-export --format csv --out compliance/iso-27001/controls-export.csv
```

## Updating evidence

When a control's implementation changes:

1. Edit `controls-mapping.yaml`: update `evidence` and `last_reviewed` fields.
2. Run `just iso-controls-validate` to check YAML schema.
3. Commit with a reference to the ADR or WP that triggered the change.

## Statement of Applicability

The Statement of Applicability (SoA) is generated from `controls-mapping.yaml`:

```bash
just iso-soa-generate --out compliance/iso-27001/soa.pdf
```

The SoA requires sign-off by the Information Security Manager before gate G5.

## Limitations

- Physical and people controls are documented in separate HR and facilities management processes
- Third-party supplier controls (data centre, HSM vendor) are covered by supplier agreements
  not included in this repository

## Related docs

- `compliance/dpa-toolkit/README.md` -- DPA s.47 security measures
- `verification/security/README.md` -- ASVS evidence referenced by controls 8.26, 8.29
- ADR-0007 -- cryptography policy
- ADR-0009 -- observability and audit logging
