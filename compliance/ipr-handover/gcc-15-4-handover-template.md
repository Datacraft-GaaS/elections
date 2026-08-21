# GCC 15.4 IPR Handover Package

Government Common Conditions (GCC) 15.4: Intellectual Property Rights
Document reference: IEBC-IPR-HANDOVER-001
Version: [VERSION]
Date: [YYYY-MM-DD]

---

## Cover Page

| Field | Value |
|---|---|
| Procuring Entity | Independent Electoral and Boundaries Commission (IEBC) |
| Supplier | [SUPPLIER_NAME] |
| Contract reference | [CONTRACT_REF] |
| Contract date | [CONTRACT_DATE] |
| Handover date | [YYYY-MM-DD] |
| Handover version | [VERSION, e.g., "v1.0.0 - pre-G5"] |
| Prepared by | [AUTHOR_NAME], [ROLE], [SUPPLIER_NAME] |
| IEBC receiving officer | [IEBC_OFFICER_NAME], [ROLE] |

---

## 1. Definitions

For the purposes of this handover package:

**Custom Software** means software developed specifically for IEBC under contract reference [CONTRACT_REF], which vests in IEBC on delivery per GCC 15.4. This includes all eel services, packages, and tools listed in Annex A.

**Standard Software** means software not developed specifically for IEBC, including open-source software incorporated into the eel platform. Standard Software is not transferred to IEBC; IEBC receives the right to use it under the applicable licence terms. A full list of Standard Software components and their licence terms is in Annex B.

**Source Code** means the complete human-readable source code of the Custom Software, including all configuration files, build scripts, and migration files necessary to build and deploy the Custom Software from source.

**Escrow** means the deposit of Source Code and associated materials with a third-party escrow agent to allow IEBC to obtain and use the Source Code in the event of supplier insolvency or material breach of contract.

**SBOM** means Software Bill of Materials: a formal, machine-readable record of all software components and their versions incorporated into the Custom Software.

---

## 2. Source Code Inventory

Full source code inventory: `compliance/ipr-handover/source-inventory.yaml`

Summary table (Custom Software only):

| Component | Repository path | Language(s) | Version |
|---|---|---|---|
| transparency-svc | services/transparency-svc/ | Rust, Elixir | [VERSION] |
| bvr-svc | services/bvr-svc/ | Rust, Python | [VERSION] |
| abis-svc | services/abis-svc/ | Rust | [VERSION] |
| bvi-svc | services/bvi-svc/ | Rust | [VERSION] |
| biometric-edge-svc | services/biometric-edge-svc/ | Rust, Flutter | [VERSION] |
| portal-svc | services/portal-svc/ | Rust, Elixir, Astro | [VERSION] |
| ecc-svc | services/ecc-svc/ | Rust, Elixir | [VERSION] |
| queue-svc | services/queue-svc/ | Rust | [VERSION] |
| inventory-svc | services/inventory-svc/ | Rust | [VERSION] |
| ai-platform-svc | services/ai-platform-svc/ | Rust, Python | [VERSION] |
| field-mobile-svc | services/field-mobile-svc/ | Rust, Flutter | [VERSION] |
| vprs-svc | services/vprs-svc/ | Rust, Elixir | [VERSION] |
| crms-svc | services/crms-svc/ | Rust, Elixir | [VERSION] |
| governance-svc | services/governance-svc/ | Rust, Python, Elixir | [VERSION] |
| training-svc | services/training-svc/ | Rust, Python, Elixir, Flutter | [VERSION] |
| support-admin-svc | services/support-admin-svc/ | Rust, Python, Elixir | [VERSION] |
| eel-crypto-rs | packages/eel-crypto-rs/ | Rust | [VERSION] |
| eel-telemetry-rs | packages/eel-telemetry-rs/ | Rust | [VERSION] |
| eel-authn-rs | packages/eel-authn-rs/ | Rust | [VERSION] |
| eel-resilience-rs | packages/eel-resilience-rs/ | Rust | [VERSION] |
| eel-outbox-rs | packages/eel-outbox-rs/ | Rust | [VERSION] |
| eel-api-server-rs | packages/eel-api-server-rs/ | Rust | [VERSION] |
| verifier-rs | packages/verifier-rs/ | Rust | [VERSION] |
| eel-completeness (tool) | tools/completeness/ | Rust | [VERSION] |
| eel-asvs-check (tool) | tools/asvs-check/ | Rust | [VERSION] |
| eel-wcag-audit (tool) | tools/wcag-audit/ | Rust | [VERSION] |

---

## 3. Licence Summary

### 3.1 Custom Software

All Custom Software listed in section 2 is licensed under Apache-2.0 and assigned to IEBC per GCC 15.4. IEBC owns the copyright and all associated intellectual property rights.

SPDX identifier: Apache-2.0

Apache-2.0 obligations applicable to IEBC as the new owner: none (IEBC may use, modify, and distribute the Custom Software without restriction under GCC 15.4; the Apache-2.0 licence continues to apply to any redistribution by IEBC to third parties).

### 3.2 Standard Software (incorporated)

Full licence obligations: `compliance/ipr-handover/software-classification.yaml` (license_obligations column)

Key Standard Software licences:

| Licence | Components | IEBC obligations |
|---|---|---|
| PostgreSQL Licence | PostgreSQL | Retain copyright notice; no copyleft |
| Apache-2.0 | Kafka, Keycloak, SpiceDB, many Rust crates | Retain NOTICE file; no copyleft |
| MIT | Many Rust crates, Dart packages | Retain copyright notice; no copyleft |
| LGPL-2.1+ | Nix | Dynamic linking permitted without copyleft |
| AGPL-3.0 | MinIO (self-hosted) | Source disclosure required if modified and served to users over network; IEBC self-hosts without modification |
| BSD-2-Clause, BSD-3-Clause | Various | Retain copyright notice |

---

## 4. Build Instructions

### 4.1 Prerequisites

- Nix package manager >= 2.18 with flakes enabled
- Docker or OCI-compatible container runtime (for testcontainers)
- x86_64-linux or aarch64-linux host (macOS supported for development)

### 4.2 Build steps

```sh
# 1. Enter the Nix development shell (pins all build tools)
nix develop

# 2. Build all Rust crates (services, packages, tools)
cargo build --release

# 3. Build all Python packages
find services -name 'pyproject.toml' -execdir uv build \;

# 4. Build all Elixir releases
find services -name 'mix.exs' -execdir mix release \;

# 5. Build all Flutter apps
find services -name 'pubspec.yaml' -execdir flutter build apk --release \;
find services -name 'pubspec.yaml' -execdir flutter build ios --release \;

# 6. Verify reproducible build (bit-identical output)
make reproduce
```

### 4.3 Running tests

```sh
# Rust tests (nextest)
cargo nextest run --profile ci

# Python tests
uv run pytest tests/ci -v

# Full integration test (requires Docker for testcontainers)
cargo nextest run --profile integration
```

### 4.4 Deployment

Deployment uses ArgoCD with Kustomize overlays. See `environments/` for environment-specific configuration. Full deployment procedure: `docs/runbooks/deploy.md`.

---

## 5. SBOM References

Signed SBOMs are generated at each release and stored in the eel git repository and the IEBC artefact registry.

| Component group | SBOM file | Format | Signature |
|---|---|---|---|
| All Rust crates | `docs/sbom/rust-sbom-[VERSION].spdx.json` | SPDX 2.3 | Ed25519 + HSM, key handle [KEY_HANDLE] |
| All Python packages | `docs/sbom/python-sbom-[VERSION].spdx.json` | SPDX 2.3 | Ed25519 + HSM |
| All container images | `docs/sbom/containers-sbom-[VERSION].spdx.json` | SPDX 2.3 | Ed25519 + HSM |
| Composite SBOM | `docs/sbom/eel-sbom-[VERSION].spdx.json` | SPDX 2.3 | Ed25519 + HSM |

To verify an SBOM signature:
```sh
# Retrieve the public key from the HSM certificate
cat docs/keys/release-signing-key-[KEY_HANDLE].pub

# Verify the signature
openssl dgst -sha256 -verify docs/keys/release-signing-key-[KEY_HANDLE].pub \
  -signature docs/sbom/eel-sbom-[VERSION].spdx.json.sig \
  docs/sbom/eel-sbom-[VERSION].spdx.json
```

---

## 6. Escrow Manifest

### 6.1 Materials deposited

The following materials are deposited with the escrow agent ([ESCROW_AGENT_NAME], [ESCROW_AGENT_ADDRESS]):

| Material | Description | Format | Date deposited |
|---|---|---|---|
| Source code archive | Complete git repository of eel at [VERSION] | git bundle (compressed) | [YYYY-MM-DD] |
| Build toolchain | Nix flake closure capturing all build dependencies | Nix export | [YYYY-MM-DD] |
| SBOM | Signed composite SBOM | SPDX 2.3 JSON | [YYYY-MM-DD] |
| Build instructions | This document section 4, plus docs/runbooks/deploy.md | PDF | [YYYY-MM-DD] |
| Test data | Synthetic voter data for build verification | CSV + SQL | [YYYY-MM-DD] |
| HSM ceremony procedure | Key restoration procedure (without actual keys) | PDF | [YYYY-MM-DD] |

### 6.2 Escrow release conditions

The escrow agent will release the deposited materials to IEBC upon receiving written notice from IEBC that any of the following conditions has occurred:

1. The supplier has been placed in liquidation, administration, receivership, or has otherwise ceased to trade.
2. The supplier has materially breached the contract and has failed to remedy the breach within 30 days of written notice.
3. The supplier has failed to provide the source code update for a period of more than [N] months after a new release has been deployed to production.
4. IEBC and the supplier have mutually agreed in writing that an escrow release is appropriate.

### 6.3 Escrow update schedule

The escrow deposit is updated within 14 days of each production release.

---

## 7. Delivery Schedule and Verification Procedure

### 7.1 Delivery milestones

| Milestone | Date | Materials |
|---|---|---|
| Initial escrow deposit | [YYYY-MM-DD] | Source code + build toolchain at G3 gate |
| G5 gate handover | [YYYY-MM-DD] | Full package at G5 gate |
| Pre-election handover | [YYYY-MM-DD] | Updated package with certifier report |
| Post-election archival | [YYYY-MM-DD] | Final update with election artefacts |

### 7.2 Verification procedure

IEBC will verify each delivery as follows:

1. Receive source code archive and extract to a clean directory.
2. Enter the Nix development shell (`nix develop`) and run `cargo build --release`.
3. Compare the SHA-256 hashes of the built artefacts with the build manifest (`reproducibility/hash-manifests/[VERSION]-manifest.yaml`). Hashes must be identical.
4. Run `cargo nextest run --profile ci` and verify that all tests pass.
5. Verify the SBOM signature using the procedure in section 5.
6. Sign the delivery acceptance form and return to the supplier.

If verification fails, IEBC will notify the supplier within 5 working days with a detailed description of the failure. The supplier must remediate within [N] working days.
