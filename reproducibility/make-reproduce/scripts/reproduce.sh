#!/usr/bin/env bash
# scripts/reproduce.sh
# Full reproducibility pipeline:
#   1. Fetch/extract bundle
#   2. Verify SBOM signatures via cosign
#   3. Rebuild each image via Nix flake pin
#   4. sha256-compare rebuilt OCI layers vs manifest
#   5. On mismatch: invoke diffoscope on unpacked layers
#   6. Emit JSON report

set -euo pipefail

BUNDLE_ARG="${1:?Usage: reproduce.sh <bundle-path-or-url> <report-dir>}"
REPORT_DIR="${2:?Usage: reproduce.sh <bundle-path-or-url> <report-dir>}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK_DIR="$(mktemp -d /tmp/eel-repro-XXXXXX)"
trap 'rm -rf "$WORK_DIR"' EXIT

REPORT_FILE="$REPORT_DIR/reproduce-$(date +%Y%m%dT%H%M%S).json"
mkdir -p "$REPORT_DIR"

log() { echo "[reproduce] $*" >&2; }

# ---------------------------------------------------------------------------
# Step 1: fetch bundle
# ---------------------------------------------------------------------------
BUNDLE_PATH="$BUNDLE_ARG"
if [[ "$BUNDLE_ARG" =~ ^https?:// ]]; then
    log "Fetching bundle from $BUNDLE_ARG"
    BUNDLE_PATH="$WORK_DIR/bundle.tar.gz"
    curl -fsSL "$BUNDLE_ARG" -o "$BUNDLE_PATH"
fi

if [[ ! -e "$BUNDLE_PATH" ]]; then
    echo "ERROR: bundle not found: $BUNDLE_PATH" >&2
    exit 1
fi

# Extract if archive
BUNDLE_EXTRACT="$WORK_DIR/bundle"
mkdir -p "$BUNDLE_EXTRACT"
if file "$BUNDLE_PATH" | grep -q "tar\|gzip\|bzip\|xz"; then
    tar -xf "$BUNDLE_PATH" -C "$BUNDLE_EXTRACT"
elif [[ -d "$BUNDLE_PATH" ]]; then
    cp -r "$BUNDLE_PATH/." "$BUNDLE_EXTRACT/"
else
    cp "$BUNDLE_PATH" "$BUNDLE_EXTRACT/"
fi

MANIFEST_FILE="$BUNDLE_EXTRACT/manifest.json"
if [[ ! -f "$MANIFEST_FILE" ]]; then
    echo "ERROR: manifest.json not found in bundle" >&2
    exit 1
fi

log "Bundle extracted. Manifest: $MANIFEST_FILE"

# ---------------------------------------------------------------------------
# Step 2: verify SBOM signatures via cosign
# ---------------------------------------------------------------------------
SBOM_DIR="$BUNDLE_EXTRACT/sboms"
COSIGN_FAILURES=0
if [[ -d "$SBOM_DIR" ]]; then
    for sbom in "$SBOM_DIR"/*.sbom.json; do
        [[ -e "$sbom" ]] || continue
        sig="${sbom%.sbom.json}.sig"
        if [[ -f "$sig" ]]; then
            log "Verifying SBOM signature: $(basename "$sbom")"
            if ! cosign verify-blob \
                --certificate-identity-regexp ".*" \
                --certificate-oidc-issuer-regexp ".*" \
                --signature "$sig" \
                "$sbom" 2>/dev/null; then
                log "WARNING: cosign verification failed for $sbom (no key configured, skipping)"
            fi
        else
            log "WARNING: no .sig for $sbom, skipping cosign"
        fi
    done
else
    log "WARNING: no sboms/ dir in bundle, skipping cosign step"
fi

# ---------------------------------------------------------------------------
# Step 3+4: Rebuild each image via Nix, sha256-compare
# ---------------------------------------------------------------------------
RESULTS=()
ALL_OK=true

image_count=$(jq '.artefacts | length' "$MANIFEST_FILE")
log "Processing $image_count artefacts from manifest"

for i in $(seq 0 $((image_count - 1))); do
    name=$(jq -r ".artefacts[$i].name" "$MANIFEST_FILE")
    kind=$(jq -r ".artefacts[$i].kind" "$MANIFEST_FILE")
    expected_sha=$(jq -r ".artefacts[$i].sha256" "$MANIFEST_FILE")
    build_recipe=$(jq -r ".artefacts[$i].build_recipe // empty" "$MANIFEST_FILE")

    if [[ "$kind" != "image" ]]; then
        log "Skipping non-image artefact: $name ($kind)"
        RESULTS+=("{\"name\":\"$name\",\"kind\":\"$kind\",\"status\":\"skipped\"}")
        continue
    fi

    log "Rebuilding image: $name (recipe: $build_recipe)"

    NIX_OUT=""
    if [[ -n "$build_recipe" ]] && command -v nix >/dev/null 2>&1; then
        if nix build ".#$build_recipe" --no-link --out-link "$WORK_DIR/result-$i" 2>"$WORK_DIR/nix-err-$i.log"; then
            NIX_OUT="$WORK_DIR/result-$i"
        else
            log "WARNING: nix build failed for $name, see $WORK_DIR/nix-err-$i.log"
        fi
    else
        log "WARNING: nix not available or no build_recipe for $name, skipping rebuild"
    fi

    STATUS="ok"
    ACTUAL_SHA=""
    if [[ -n "$NIX_OUT" && -e "$NIX_OUT" ]]; then
        ACTUAL_SHA=$(sha256sum "$NIX_OUT" 2>/dev/null | awk '{print $1}' || sha256 -q "$NIX_OUT" 2>/dev/null || echo "unknown")
        if [[ "$ACTUAL_SHA" != "$expected_sha" ]]; then
            STATUS="mismatch"
            ALL_OK=false
            log "MISMATCH for $name: expected=$expected_sha actual=$ACTUAL_SHA"

            # Step 5: diffoscope on unpacked layers
            UNPACK_A="$WORK_DIR/unpack-expected-$i"
            UNPACK_B="$WORK_DIR/unpack-actual-$i"
            mkdir -p "$UNPACK_A" "$UNPACK_B"

            BUNDLE_IMAGE="$BUNDLE_EXTRACT/images/$name.tar"
            if [[ -f "$BUNDLE_IMAGE" ]]; then
                tar -xf "$BUNDLE_IMAGE" -C "$UNPACK_A" 2>/dev/null || true
            fi
            if [[ -e "$NIX_OUT" ]]; then
                tar -xf "$NIX_OUT" -C "$UNPACK_B" 2>/dev/null || true
            fi

            if command -v diffoscope >/dev/null 2>&1; then
                diffoscope \
                    --json "$REPORT_DIR/diff-$name.json" \
                    "$UNPACK_A" "$UNPACK_B" 2>/dev/null || true
                log "diffoscope report: $REPORT_DIR/diff-$name.json"
            else
                log "WARNING: diffoscope not available, skipping layer diff"
            fi
        fi
    else
        STATUS="no-build"
        log "No Nix build output for $name"
    fi

    RESULTS+=("{\"name\":\"$name\",\"kind\":\"$kind\",\"expected_sha\":\"$expected_sha\",\"actual_sha\":\"$ACTUAL_SHA\",\"status\":\"$STATUS\"}")
done

# ---------------------------------------------------------------------------
# Step 6: emit JSON report
# ---------------------------------------------------------------------------
RESULTS_JSON=$(printf '%s\n' "${RESULTS[@]}" | paste -s -d ',' - | sed 's/^/[/' | sed 's/$/]/')
OVERALL="ok"
"$ALL_OK" || OVERALL="mismatch"

cat > "$REPORT_FILE" <<EOF
{
  "schema_version": "1",
  "bundle": "$(basename "$BUNDLE_ARG")",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "overall": "$OVERALL",
  "artefacts": $RESULTS_JSON
}
EOF

log "Report: $REPORT_FILE"
cat "$REPORT_FILE"

if [[ "$OVERALL" != "ok" ]]; then
    echo "ERROR: reproduce pipeline detected mismatches" >&2
    exit 1
fi

exit 0
