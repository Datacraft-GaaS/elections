#!/usr/bin/env bash
# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

set -euo pipefail

# verify-submission.sh
# Usage: verify-submission.sh <submission.tar.gz> <reference.tar.gz> <output-dir>
#
# Exit codes:
#   0  submission matches reference exactly
#   1  diff present (files differ)
#   2  manifest.json missing from submission
#   3  SHA-256 mismatch between manifest and actual file

SUBMISSION_TARBALL="${1:-}"
REFERENCE_TARBALL="${2:-}"
OUTPUT_DIR="${3:-}"

# ---------------------------------------------------------------------------
# Argument validation
# ---------------------------------------------------------------------------

if [[ -z "$SUBMISSION_TARBALL" || -z "$REFERENCE_TARBALL" || -z "$OUTPUT_DIR" ]]; then
	echo "ERROR: Usage: $0 <submission.tar.gz> <reference.tar.gz> <output-dir>" >&2
	exit 2
fi

if [[ ! -f "$SUBMISSION_TARBALL" ]]; then
	echo "ERROR: Submission tarball not found: $SUBMISSION_TARBALL" >&2
	exit 2
fi

if [[ ! -f "$REFERENCE_TARBALL" ]]; then
	echo "ERROR: Reference bundle not found: $REFERENCE_TARBALL" >&2
	exit 2
fi

# ---------------------------------------------------------------------------
# Setup
# ---------------------------------------------------------------------------

WORK_DIR=$(mktemp -d)
trap 'rm -rf "$WORK_DIR"' EXIT

SUB_DIR="$WORK_DIR/submission"
REF_DIR="$WORK_DIR/reference"
mkdir -p "$SUB_DIR" "$REF_DIR" "$OUTPUT_DIR"

TEAM_ID="unknown"
DAY="unknown"
RESULT="FAIL"
DIFF_OUTPUT=""
SUBMISSION_SHA256=$(sha256sum "$SUBMISSION_TARBALL" | awk '{print $1}')
REFERENCE_SHA256=$(sha256sum "$REFERENCE_TARBALL" | awk '{print $1}')
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# ---------------------------------------------------------------------------
# Extract tarballs
# ---------------------------------------------------------------------------

echo "Extracting submission tarball..."
tar -xzf "$SUBMISSION_TARBALL" -C "$SUB_DIR"

echo "Extracting reference tarball..."
tar -xzf "$REFERENCE_TARBALL" -C "$REF_DIR"

# ---------------------------------------------------------------------------
# Validate manifest.json
# ---------------------------------------------------------------------------

MANIFEST="$SUB_DIR/manifest.json"

if [[ ! -f "$MANIFEST" ]]; then
	echo "ERROR: manifest.json not found in submission" >&2
	RESULT="FAIL"
	DIFF_OUTPUT="manifest.json missing from submission tarball"
	# Render minimal report before exiting
	TEAM_ID="unknown"
	DAY="unknown"
	sed \
		-e "s|{{RESULT}}|$RESULT|g" \
		-e "s|{{TEAM_ID}}|$TEAM_ID|g" \
		-e "s|{{DAY}}|$DAY|g" \
		-e "s|{{SUBMISSION_SHA256}}|$SUBMISSION_SHA256|g" \
		-e "s|{{REFERENCE_SHA256}}|$REFERENCE_SHA256|g" \
		-e "s|{{TIMESTAMP}}|$TIMESTAMP|g" \
		-e "s|{{DIFF_OUTPUT}}|$DIFF_OUTPUT|g" \
		"$(dirname "$0")/diff-report-template.md.tmpl" \
		> "$OUTPUT_DIR/report.md"
	exit 2
fi

# Extract team_id and day from manifest
TEAM_ID=$(python3 -c "import json,sys; d=json.load(open('$MANIFEST')); print(d.get('team_id','unknown'))")
DAY=$(python3 -c "import json,sys; d=json.load(open('$MANIFEST')); print(d.get('day','unknown'))")

echo "Team: $TEAM_ID  Day: $DAY"

# ---------------------------------------------------------------------------
# SHA-256 verification against manifest
# ---------------------------------------------------------------------------

echo "Verifying SHA-256 checksums from manifest..."

SHA_FAIL=0
python3 - "$SUB_DIR" "$MANIFEST" <<'PYEOF'
import json, hashlib, sys, pathlib

sub_dir = pathlib.Path(sys.argv[1])
manifest = json.loads(pathlib.Path(sys.argv[2]).read_text())

for entry in manifest.get("files", []):
    path = sub_dir / entry["path"]
    expected = entry["sha256"]
    if not path.exists():
        print(f"MISSING: {entry['path']}", flush=True)
        sys.exit(3)
    actual = hashlib.sha256(path.read_bytes()).hexdigest()
    if actual != expected:
        print(f"SHA_MISMATCH: {entry['path']}  expected={expected}  actual={actual}", flush=True)
        sys.exit(3)
    print(f"OK: {entry['path']}")
PYEOF
SHA_RESULT=$?

if [[ $SHA_RESULT -ne 0 ]]; then
	echo "ERROR: SHA-256 mismatch detected" >&2
	RESULT="FAIL"
	DIFF_OUTPUT="SHA-256 mismatch in one or more manifest-listed files"
	sed \
		-e "s|{{RESULT}}|$RESULT|g" \
		-e "s|{{TEAM_ID}}|$TEAM_ID|g" \
		-e "s|{{DAY}}|$DAY|g" \
		-e "s|{{SUBMISSION_SHA256}}|$SUBMISSION_SHA256|g" \
		-e "s|{{REFERENCE_SHA256}}|$REFERENCE_SHA256|g" \
		-e "s|{{TIMESTAMP}}|$TIMESTAMP|g" \
		-e "s|{{DIFF_OUTPUT}}|$DIFF_OUTPUT|g" \
		"$(dirname "$0")/diff-report-template.md.tmpl" \
		> "$OUTPUT_DIR/report.md"
	exit 3
fi

# ---------------------------------------------------------------------------
# Diff submission vs reference
# ---------------------------------------------------------------------------

echo "Running diff against reference bundle..."

DIFF_OUTPUT=$(diff -rq "$SUB_DIR" "$REF_DIR" 2>&1 || true)

if [[ -z "$DIFF_OUTPUT" ]]; then
	RESULT="PASS"
	EXIT_CODE=0
	echo "Result: PASS"
else
	RESULT="FAIL"
	EXIT_CODE=1
	echo "Result: FAIL"
	echo "$DIFF_OUTPUT"
fi

# ---------------------------------------------------------------------------
# Render report
# ---------------------------------------------------------------------------

# Escape any sed-special characters in DIFF_OUTPUT
DIFF_ESCAPED=$(printf '%s' "$DIFF_OUTPUT" | sed 's/[&/\]/\\&/g; s/$/\\n/' | tr -d '\n')
# Remove the trailing \n added for the last line
DIFF_ESCAPED="${DIFF_ESCAPED%\\n}"

TEMPLATE="$(dirname "$0")/diff-report-template.md.tmpl"

sed \
	-e "s|{{RESULT}}|$RESULT|g" \
	-e "s|{{TEAM_ID}}|$TEAM_ID|g" \
	-e "s|{{DAY}}|$DAY|g" \
	-e "s|{{SUBMISSION_SHA256}}|$SUBMISSION_SHA256|g" \
	-e "s|{{REFERENCE_SHA256}}|$REFERENCE_SHA256|g" \
	-e "s|{{TIMESTAMP}}|$TIMESTAMP|g" \
	-e "s|{{DIFF_OUTPUT}}|$DIFF_ESCAPED|g" \
	"$TEMPLATE" \
	> "$OUTPUT_DIR/report.md"

echo "Report written to $OUTPUT_DIR/report.md"
exit $EXIT_CODE
