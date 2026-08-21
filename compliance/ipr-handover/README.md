<title>IPR and Source Handover</title>

# Intellectual Property and Source Handover

Process and documentation for transferring intellectual property and source code to the Commission
upon project completion, per GCC clause 15.4 and the Elections (Technology) Regulations 2017
Regulation 23 (source code escrow).

## Purpose

WP-5.14. Ensures that the Commission receives full ownership of all deliverables, can independently
maintain and audit the system, and that a third-party escrow holds source code for continuity.

## What is transferred

| Asset | Form | Timing |
|-------|------|--------|
| Full monorepo source code | Git bundle (signed) | At each gate + final |
| Build toolchain and justfile recipes | Included in repo | Continuous |
| Container image build configs | Dockerfile + CI scripts | Continuous |
| Deployment manifests (Kustomize) | `deploy/` per service | Continuous |
| Cryptographic key material | HSM export + OpenBao backup | Final handover only |
| Training materials | `services/training-svc/` | At G4 |
| Operational runbooks | `docs/runbooks/` | At G5 |
| Compliance evidence packages | `compliance/` + `verification/reports/` | At G6 |
| Translation sign-offs | `translation-signoffs.yaml` | At G5 |
| Third-party licence schedule | `ipr-schedule.yaml` | At G5 |

## Licence and ownership

All code written by the delivery team is assigned to the Commission under the contract. The
`ipr-schedule.yaml` file enumerates every third-party open-source dependency with its licence,
confirming no copyleft licence incompatibility with Commission ownership.

```bash
# Generate licence schedule from Cargo.lock, uv.lock, pubspec.lock, package-lock.json
just ipr-licence-scan --out compliance/ipr-handover/ipr-schedule.yaml
```

## Source code escrow

Per ETR Regulation 23, source code is deposited with an approved escrow agent:

1. Generate a signed Git bundle at each gate:

```bash
just ipr-bundle --gate G5 --out compliance/ipr-handover/bundles/eel-G5.bundle
# Bundle is signed with the project GPG key; SHA-256 hash appended to transparency log
```

2. Upload to escrow agent (NCC Group or Iron Mountain Kenya):

```bash
just ipr-escrow-upload --bundle compliance/ipr-handover/bundles/eel-G5.bundle \
  --agent iron-mountain --reference ESCROW-2026-001
```

3. Record receipt in `escrow-receipts.yaml`.

## Handover verification

The Commission's technical team should be able to:

```bash
# Verify the Git bundle signature and extract
git bundle verify compliance/ipr-handover/bundles/eel-G6.bundle
git clone compliance/ipr-handover/bundles/eel-G6.bundle eel-handover

# Build all services from source (no internet required after deps cached)
cd eel-handover && just build-all

# Run full CI suite
just ci-all
```

Passing these steps constitutes acceptance of the handover artefact.

## Translation sign-offs

`translation-signoffs.yaml` records sign-off by native-speaker reviewers for all Kiswahili and
Somali user-facing strings. Required for acceptance of portal-svc, kit, and field-mobile-svc.

## Final handover checklist

- [ ] All gate evidence signed and appended to transparency log
- [ ] `ipr-schedule.yaml` complete with all third-party licences
- [ ] Escrow receipts for G1 through G6 bundles in `escrow-receipts.yaml`
- [ ] Commission's technical team has completed handover verification (`just build-all` passes)
- [ ] HSM key material transferred under dual-control procedure (separate operational process)
- [ ] All `docs/runbooks/` reviewed and signed off by Commission operations team
- [ ] Translation sign-offs complete in `translation-signoffs.yaml`
- [ ] Knowledge transfer sessions completed (attendance sheets in `ipr-handover/kt-sessions/`)

## Limitations

- HSM key export and import is a physical dual-control process not automated here
- Escrow upload requires network access to the escrow agent's SFTP endpoint
- Licence compatibility opinion for GPL-adjacent licences should be confirmed by legal counsel

## Related docs

- `compliance/elections-tech-regs/README.md` -- ETR Regulation 23
- `compliance/README.md` -- compliance programme overview
- `docs/runbooks/` -- operational runbooks included in handover
- ADR-0010 -- audit retention and evidence package requirements
