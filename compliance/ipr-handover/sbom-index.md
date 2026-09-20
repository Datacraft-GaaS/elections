# SBOM Index

Software Bill of Materials for all eel service images. SBOMs are produced by the CI supply-chain pipeline (WP-0.5) using Syft and attested via Cosign.

## Location convention

Each OCI image has a corresponding SBOM stored in the container registry as an attestation:

```
<registry>/<service>:<tag>.sbom.spdx.json
```

The SBOM attestation is verifiable with Cosign:

```sh
cosign verify-attestation \
  --type spdxjson \
  --certificate-identity "https://github.com/eel/eel/.github/workflows/supply-chain.yaml" \
  <registry>/<service>:<tag>
```

## Service SBOM Links

SBOMs are generated per release. The table below shows the SBOM path pattern; replace `<release>` with the tag from the current release manifest in `reproducibility/hash-manifests/`.

| Service | SBOM path pattern | Format |
|---|---|---|
| transparency-svc | `<registry>/transparency-svc:<release>.sbom.spdx.json` | SPDX 2.3 |
| boundary-svc | `<registry>/boundary-svc:<release>.sbom.spdx.json` | SPDX 2.3 |
| rts-svc | `<registry>/rts-svc:<release>.sbom.spdx.json` | SPDX 2.3 |
| bvr-svc | `<registry>/bvr-svc:<release>.sbom.spdx.json` | SPDX 2.3 |
| abis-svc | `<registry>/abis-svc:<release>.sbom.spdx.json` | SPDX 2.3 |
| bvi-svc | `<registry>/bvi-svc:<release>.sbom.spdx.json` | SPDX 2.3 |
| biometric-edge-svc | `<registry>/biometric-edge-svc:<release>.sbom.spdx.json` | SPDX 2.3 |
| portal-svc | `<registry>/portal-svc:<release>.sbom.spdx.json` | SPDX 2.3 |
| ecc-svc | `<registry>/ecc-svc:<release>.sbom.spdx.json` | SPDX 2.3 |
| queue-svc | `<registry>/queue-svc:<release>.sbom.spdx.json` | SPDX 2.3 |
| inventory-svc | `<registry>/inventory-svc:<release>.sbom.spdx.json` | SPDX 2.3 |
| ai-platform-svc | `<registry>/ai-platform-svc:<release>.sbom.spdx.json` | SPDX 2.3 |
| field-mobile-svc | `<registry>/field-mobile-svc:<release>.sbom.spdx.json` | SPDX 2.3 |
| vprs-svc | `<registry>/vprs-svc:<release>.sbom.spdx.json` | SPDX 2.3 |
| crms-svc | `<registry>/crms-svc:<release>.sbom.spdx.json` | SPDX 2.3 |
| governance-svc | `<registry>/governance-svc:<release>.sbom.spdx.json` | SPDX 2.3 |
| training-svc | `<registry>/training-svc:<release>.sbom.spdx.json` | SPDX 2.3 |
| support-admin-svc | `<registry>/support-admin-svc:<release>.sbom.spdx.json` | SPDX 2.3 |

## Shared Package SBOMs

All shared packages (eel-auth-*, eel-crypto-rs, eel-events-*, etc.) are included in the consuming service SBOM via Syft workspace scanning. Standalone SBOM attestations are also available for each package image.

## Vendored components

Non-registry code vendored into source trees. Syft records these as
`vendored` files in the consuming service's SBOM; the table is the auditable
index of *what* is vendored, where it is pinned, and under which license.

| Component | Version / pinned SHA | License | Location | Consumed by |
|---|---|---|---|---|
| NIST Biometric Image Software (NBIS) | v5.0.0 @ `c946acc42dbfca7090d00b1098f2b93a04ba8914` | Public domain (US federal government work) | `services/biometric-edge-svc/crates/nearform-nbis/vendor/nbis/` (396 files, 7 subtrees; see `vendor/README.provenance`) | `nearform-nbis` → `biometric-edge-svc` (fingerprint extraction + Bozorth3 matching) |

## SBOM Tooling

- Generator: Syft v1.x (pinned in `tools/supply-chain/Dockerfile`)
- Format: SPDX 2.3 JSON
- Attestation: Cosign with Sigstore transparency log
- CI integration: `tools/supply-chain/generate-sbom.sh` runs after every successful image build

## Verification

```sh
# List all SBOMs for a release
just sbom-list <release>

# Verify a specific SBOM attestation
just sbom-verify <service> <release>

# Merge SBOMs from all services into a workspace SBOM
just sbom-merge <release>
```

## Related

- `compliance/ipr-handover/licence-report.md` - third-party dependency licence summary
- `tools/supply-chain/` - SBOM generation pipeline
- WP-0.5 gate report: `compliance/ipr-handover/provenance/wave-0.yaml`
