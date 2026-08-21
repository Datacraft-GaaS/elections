# air-gapped-build/sbom-cache

Storage for Software Bill of Materials (SBOM) artifacts produced by eel builds. Every
service image produces a CycloneDX or SPDX SBOM at build time; this directory caches
those SBOMs as input to hash manifests and compliance evidence packages.

## Purpose

SBOMs document every dependency included in a build artifact: library name, version, license,
and provenance. For the eel tender, SBOMs are required evidence for:

- Supply chain security review (what third-party software runs on election day)
- License compliance (open source license inventory)
- Vulnerability scanning (CVE matching against the SBOM dependency list)
- Hash manifest cross-reference (each manifest entry links to its `sbom_ref`)

## Contents

SBOMs are stored per artifact, named by the artifact name and build hash:

```
sbom-cache/
  eel-rts-svc-<sha256-prefix>.cdx.json        # CycloneDX JSON
  eel-rts-svc-<sha256-prefix>.spdx.json       # SPDX JSON (if generated)
  eel-portal-svc-<sha256-prefix>.cdx.json
  ...
  index.json                                   # Index mapping artifact name to SBOM paths
```

## Usage

```bash
# SBOMs are generated automatically during nix build via the syft post-build hook.
# To manually generate an SBOM for a built output:
syft packages "$(nix path-info .#eel-rts-svc)" \
  -o cyclonedx-json \
  > sbom-cache/eel-rts-svc-$(nix path-info .#eel-rts-svc | sha256sum | cut -c1-16).cdx.json

# List cached SBOMs
ls -1 sbom-cache/*.cdx.json

# Scan for known vulnerabilities
grype sbom:sbom-cache/eel-rts-svc-<hash>.cdx.json
```

## Config and environment

| Variable | Description |
|----------|-------------|
| `SBOM_FORMAT` | `cyclonedx-json` (default) or `spdx-json`; set in build hooks |
| `SBOM_CACHE_DIR` | Path to this directory; referenced by `build.sh` and `reproduce.sh` |

## Dependencies

- `syft` for SBOM generation (included in the build-image container)
- `grype` for vulnerability scanning against generated SBOMs (optional, CI use)
- `jq` for `index.json` maintenance

## Test and verify

```bash
# Validate SBOM schema (CycloneDX)
curl -s https://raw.githubusercontent.com/CycloneDX/specification/master/schema/bom-1.5.schema.json \
  -o /tmp/cdx-schema.json
# (offline: use pre-downloaded schema)
python3 -c "
import json, jsonschema, sys
schema = json.load(open('/tmp/cdx-schema.json'))
sbom   = json.load(open('sbom-cache/eel-rts-svc-example.cdx.json'))
jsonschema.validate(sbom, schema)
print('SBOM valid')
"
```

## Related docs

- `air-gapped-build/README.md`: overall offline build workflow
- `air-gapped-build/timestamps/README.md`: RFC 3161 timestamps for SBOMs
- `reproducibility/hash-manifests/README.md`: how `sbom_ref` fields link to this cache

## Limitations

- SBOMs are not generated for intermediate build steps, only final artifact outputs.
- The `index.json` is not automatically updated; the build hooks must be configured to
  append entries on each build.
- Vulnerability scan results from `grype` are not stored here; use the CI pipeline
  artifact store for scan reports.
