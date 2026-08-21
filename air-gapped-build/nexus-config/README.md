# air-gapped-build/nexus-config

Sonatype Nexus OSS bootstrap and pinned repository configuration for the eel offline build
environment. These files provision all proxy repositories that the build image needs,
with strict checksum enforcement on every package.

## Purpose

Before airgapping the build environment, an operator runs the Nexus bootstrap script to
create proxy repositories for every package ecosystem used by eel: Rust (crates.io), Python
(PyPI), JavaScript (npm), Elixir (Hex.pm), Dart (pub.dev), and Nix (binary cache). All
proxies are configured with `STRICT` checksum policy so that tampered or corrupted packages
are rejected at fetch time.

## Contents

| File | Description |
|------|-------------|
| `nexus.groovy` | Groovy bootstrap script: creates all proxy repositories via Nexus scripting API |
| `pinned-repos.yaml` | Declarative list of all repositories with upstream URLs and checksum policy |

## Repositories provisioned

| Nexus repo name | Format | Upstream |
|----------------|--------|----------|
| `eel-crates-io-proxy` | raw | `https://static.crates.io` |
| `eel-pypi-proxy` | pypi | `https://pypi.org` |
| `eel-npm-proxy` | npm | `https://registry.npmjs.org` |
| `eel-hex-proxy` | raw | `https://repo.hex.pm` |
| `eel-pub-proxy` | raw | `https://pub.dev` |
| `eel-nix-cache-proxy` | raw | `https://cache.nixos.org` |
| `eel-docker-proxy` | docker | `https://registry-1.docker.io` |

All repos default: `content_max_age: 1440min`, `metadata_max_age: 60min`,
`checksum_policy: STRICT`, `negative_cache_ttl: 60min`.

## Usage

```bash
# Step 1: upload the bootstrap script via Nexus scripting API
curl -u admin:$NEXUS_PASS \
  -H "Content-Type: application/json" \
  http://nexus.internal:8081/service/rest/v1/script \
  --data-binary @<(jq -n \
    --rawfile content nexus.groovy \
    '{"name":"eel-nexus-bootstrap","type":"groovy","content":$content}')

# Step 2: run the bootstrap script
curl -u admin:$NEXUS_PASS -X POST \
  http://nexus.internal:8081/service/rest/v1/script/eel-nexus-bootstrap/run

# Step 3: verify repositories were created
curl -s -u admin:$NEXUS_PASS \
  http://nexus.internal:8081/service/rest/v1/repositories \
  | jq '.[].name' | grep eel-
```

## Config and environment

| Variable | Description |
|----------|-------------|
| `NEXUS_URL` | Nexus base URL (default `http://nexus.internal:8081`) |
| `NEXUS_PASS` | Admin password; retrieve from vault before running |
| `NEXUS_CA` | CA bundle path for HTTPS Nexus; leave unset for HTTP-only setups |

## Dependencies

- Sonatype Nexus OSS >= 3.60 running and reachable
- `curl` and `jq` to run the provisioning commands
- Internet access at bootstrap time (proxy population); no internet needed after airgapping

## Test and verify

```bash
# Confirm all expected repos exist
EXPECTED="eel-crates-io-proxy eel-pypi-proxy eel-npm-proxy eel-hex-proxy eel-pub-proxy eel-nix-cache-proxy eel-docker-proxy"
ACTUAL=$(curl -s -u admin:$NEXUS_PASS \
  http://nexus.internal:8081/service/rest/v1/repositories \
  | jq -r '.[].name' | tr '\n' ' ')
for r in $EXPECTED; do
  echo "$ACTUAL" | grep -q "$r" && echo "OK: $r" || echo "MISSING: $r"
done

# Confirm checksum policy is STRICT on all eel- repos
curl -s -u admin:$NEXUS_PASS \
  http://nexus.internal:8081/service/rest/v1/repositories \
  | jq '.[] | select(.name | startswith("eel-")) | {name, checksum: .proxy.checksumPolicy}'
```

## Related docs

- `air-gapped-build/README.md`: overall offline build workflow
- `air-gapped-build/build-image/README.md`: how build scripts consume the Nexus proxies
- `docs/adr/0006-reproducible-builds.md`: design ADR

## Limitations

- Re-running the bootstrap script is idempotent (it skips existing repos), but does not
  update existing repo configuration. Modify repos via the Nexus UI or API directly.
- `eel-nix-cache-proxy` stores binary cache `.narinfo` and `.nar` files; these can be
  large (10GB+ for a full wave). Allocate Nexus blob store accordingly.
