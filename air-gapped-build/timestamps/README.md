# air-gapped-build/timestamps

RFC 3161 trusted timestamps for eel build artifacts. Each artifact produced in a release
build receives a timestamp token from a qualified Timestamp Authority (TSA), providing
legally-binding proof of when the artifact existed.

## Purpose

Timestamps serve two purposes in the eel evidentiary chain:

1. Prove that artifacts were built before election day (not fabricated after the fact).
2. Satisfy the tender's non-repudiation requirement: the Commission can demonstrate that
   reviewed source code produced a specific binary at a specific time, signed by an
   independent TSA.

Timestamp tokens are stored alongside SBOMs and are referenced in hash manifests.

## Timestamp workflow

```
artifact sha256 -> RFC 3161 request -> TSA -> timestamp token (.tsr)
                                             -> verified and stored here
```

The `stamp.sh` script sends a timestamp request for each artifact in the hash manifest
to the configured TSA endpoint. Tokens are stored as `<artifact-name>-<hash>.tsr` files.

## Contents

```
timestamps/
  stamp.sh                    # Timestamps a file; stores .tsr token here
  verify.sh                   # Verifies a .tsr token against a file
  tsa-ca.pem                  # TSA CA certificate bundle for verification
  eel-rts-svc-<hash>.tsr      # Example token (generated, not checked in as binary)
  index.json                  # Maps artifact names to .tsr paths
```

## Usage

```bash
# Timestamp a build artifact
cd air-gapped-build/timestamps
bash stamp.sh /nix/store/...-eel-rts-svc/bin/eel-rts-svc

# Verify a timestamp token
bash verify.sh \
  /nix/store/...-eel-rts-svc/bin/eel-rts-svc \
  timestamps/eel-rts-svc-<hash>.tsr

# Batch timestamp all artifacts in a hash manifest
jq -r '.artefacts[].sha256' /path/to/manifest.json | while read sha; do
  bash stamp.sh "$sha"
done
```

## Config and environment

| Variable | Default | Description |
|----------|---------|-------------|
| `TSA_URL` | required | RFC 3161 TSA endpoint (e.g., a qualified EU TSA) |
| `TSA_CA` | `tsa-ca.pem` | CA bundle for verifying TSA responses |
| `OPENSSL_BIN` | `openssl` | OpenSSL binary for RFC 3161 operations |

For air-gapped builds where the TSA is not reachable, timestamp requests are batched and
sent via a network jump host. The `.tsq` (request) files are generated offline and the
`.tsr` (response) files are transferred back.

## Dependencies

- `openssl` >= 3.0 for RFC 3161 timestamp request/response operations
- A qualified Timestamp Authority accessible at `$TSA_URL` (online step; can be deferred
  for airgapped builds using the batch workflow above)

## Test and verify

```bash
# Verify the example timestamp token
openssl ts -verify \
  -in timestamps/eel-rts-svc-example.tsr \
  -data /path/to/eel-rts-svc-example-binary \
  -CAfile timestamps/tsa-ca.pem
# Expected: Verification: OK
```

## Related docs

- `air-gapped-build/README.md`: overall offline build workflow
- `air-gapped-build/sbom-cache/README.md`: SBOMs that are also timestamped
- `reproducibility/hash-manifests/README.md`: where timestamp paths are referenced

## Limitations

- RFC 3161 requires internet access to reach the TSA; in fully airgapped environments,
  the batch deferred workflow adds manual steps.
- Token format is binary DER (`.tsr`); do not open in a text editor. Use `openssl ts -reply -in token.tsr -text` to inspect.
- TSA certificate expiry must be monitored; expired TSA certs do not invalidate old
  timestamps but do prevent new ones.
