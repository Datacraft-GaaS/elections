#!/usr/bin/env bash
set -euo pipefail

# test_verify.sh - Integration tests for verify-submission.sh
#
# Supports bats (if installed) or standalone plain-bash mode.
# Run: bash tests/test_verify.sh
#       bats tests/test_verify.sh  (if bats is available)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERIFY="$SCRIPT_DIR/../verify-submission.sh"
FIXTURES="$SCRIPT_DIR/../fixtures/mini-submission"

# Real SHA-256 values for the fixture files
TALLY_SHA256="b760c3d519f205ab48e3a604af21212057baf11ce61403550dc17ae7ec197205"
EVENTS_SHA256="c802ff599e450d5af62a402e025986025bbeb9eced53561eecf70e707c223a81"

# ---------------------------------------------------------------------------
# Plain-bash test harness (used when bats is absent)
# ---------------------------------------------------------------------------

_PASS=0
_FAIL=0
_FAILURES=()

_run_test() {
	local name="$1"
	local fn="$2"
	if "$fn"; then
		echo "  PASS  $name"
		(( _PASS++ )) || true
	else
		echo "  FAIL  $name"
		(( _FAIL++ )) || true
		_FAILURES+=("$name")
	fi
}

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

_make_submission_tarball() {
	# Args: dest_tarball.tar.gz  source_dir
	local dest="$1"
	local src="$2"
	tar -czf "$dest" -C "$src" .
}

_make_manifest() {
	# Args: dir  tally_sha  events_sha
	local dir="$1"
	local tsha="${2:-$TALLY_SHA256}"
	local esha="${3:-$EVENTS_SHA256}"
	cat > "$dir/manifest.json" <<EOF
{
  "files": [
    {"path": "tally.json",    "sha256": "$tsha"},
    {"path": "events.ndjson", "sha256": "$esha"}
  ],
  "day":     "2024-08-01",
  "team_id": "eel-test-team"
}
EOF
}

_copy_fixtures() {
	local dir="$1"
	cp "$FIXTURES/tally.json"     "$dir/tally.json"
	cp "$FIXTURES/events.ndjson"  "$dir/events.ndjson"
}

# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------

# Test 1: happy path - matching submission passes (exit 0)
_test_happy_path() {
	local tmp
	tmp=$(mktemp -d)
	trap 'rm -rf "$tmp"' RETURN
	local sub_dir="$tmp/sub"  ref_dir="$tmp/ref"  out_dir="$tmp/out"
	mkdir -p "$sub_dir" "$ref_dir"
	_copy_fixtures "$sub_dir"
	_copy_fixtures "$ref_dir"
	_make_manifest "$sub_dir"
	_make_manifest "$ref_dir"
	_make_submission_tarball "$tmp/submission.tar.gz" "$sub_dir"
	_make_submission_tarball "$tmp/reference.tar.gz"  "$ref_dir"
	bash "$VERIFY" "$tmp/submission.tar.gz" "$tmp/reference.tar.gz" "$out_dir"
}

# Test 2: tampered tally.json causes diff (exit 1)
_test_tampered_tally() {
	local tmp
	tmp=$(mktemp -d)
	trap 'rm -rf "$tmp"' RETURN
	local sub_dir="$tmp/sub"  ref_dir="$tmp/ref"  out_dir="$tmp/out"
	mkdir -p "$sub_dir" "$ref_dir"
	_copy_fixtures "$sub_dir"
	_copy_fixtures "$ref_dir"
	# Tamper sub tally but keep original SHA in manifest so SHA check passes
	# We need the manifest SHA to match the ACTUAL tampered file to pass SHA check
	echo '{"stations":3,"total_votes":999}' > "$sub_dir/tally.json"
	local tampered_sha
	tampered_sha=$(sha256sum "$sub_dir/tally.json" | awk '{print $1}')
	_make_manifest "$sub_dir" "$tampered_sha" "$EVENTS_SHA256"
	_make_manifest "$ref_dir"
	_make_submission_tarball "$tmp/submission.tar.gz" "$sub_dir"
	_make_submission_tarball "$tmp/reference.tar.gz"  "$ref_dir"
	local rc=0
	bash "$VERIFY" "$tmp/submission.tar.gz" "$tmp/reference.tar.gz" "$out_dir" || rc=$?
	[[ $rc -eq 1 ]]
}

# Test 3: missing manifest.json rejected (exit 2)
_test_missing_manifest() {
	local tmp
	tmp=$(mktemp -d)
	trap 'rm -rf "$tmp"' RETURN
	local sub_dir="$tmp/sub"  ref_dir="$tmp/ref"  out_dir="$tmp/out"
	mkdir -p "$sub_dir" "$ref_dir"
	_copy_fixtures "$sub_dir"
	_copy_fixtures "$ref_dir"
	_make_manifest "$ref_dir"
	# No manifest in sub_dir
	_make_submission_tarball "$tmp/submission.tar.gz" "$sub_dir"
	_make_submission_tarball "$tmp/reference.tar.gz"  "$ref_dir"
	local rc=0
	bash "$VERIFY" "$tmp/submission.tar.gz" "$tmp/reference.tar.gz" "$out_dir" || rc=$?
	[[ $rc -eq 2 ]]
}

# Test 4: wrong SHA-256 in manifest rejected (exit 3)
_test_wrong_sha() {
	local tmp
	tmp=$(mktemp -d)
	trap 'rm -rf "$tmp"' RETURN
	local sub_dir="$tmp/sub"  ref_dir="$tmp/ref"  out_dir="$tmp/out"
	mkdir -p "$sub_dir" "$ref_dir"
	_copy_fixtures "$sub_dir"
	_copy_fixtures "$ref_dir"
	_make_manifest "$sub_dir" "deadbeef000000000000000000000000000000000000000000000000deadbeef" "$EVENTS_SHA256"
	_make_manifest "$ref_dir"
	_make_submission_tarball "$tmp/submission.tar.gz" "$sub_dir"
	_make_submission_tarball "$tmp/reference.tar.gz"  "$ref_dir"
	local rc=0
	bash "$VERIFY" "$tmp/submission.tar.gz" "$tmp/reference.tar.gz" "$out_dir" || rc=$?
	[[ $rc -eq 3 ]]
}

# Test 5: empty tarball rejected (exit 2 - manifest missing)
_test_empty_tarball() {
	local tmp
	tmp=$(mktemp -d)
	trap 'rm -rf "$tmp"' RETURN
	local ref_dir="$tmp/ref"  out_dir="$tmp/out"
	mkdir -p "$tmp/empty" "$ref_dir"
	_copy_fixtures "$ref_dir"
	_make_manifest "$ref_dir"
	tar -czf "$tmp/submission.tar.gz" -C "$tmp/empty" . 2>/dev/null || tar -czf "$tmp/submission.tar.gz" -T /dev/null
	_make_submission_tarball "$tmp/reference.tar.gz" "$ref_dir"
	local rc=0
	bash "$VERIFY" "$tmp/submission.tar.gz" "$tmp/reference.tar.gz" "$out_dir" || rc=$?
	[[ $rc -eq 2 ]]
}

# Test 6: missing reference bundle rejected (exit 2)
_test_missing_reference() {
	local tmp
	tmp=$(mktemp -d)
	trap 'rm -rf "$tmp"' RETURN
	local sub_dir="$tmp/sub"  out_dir="$tmp/out"
	mkdir -p "$sub_dir"
	_copy_fixtures "$sub_dir"
	_make_manifest "$sub_dir"
	_make_submission_tarball "$tmp/submission.tar.gz" "$sub_dir"
	local rc=0
	bash "$VERIFY" "$tmp/submission.tar.gz" "$tmp/nonexistent-reference.tar.gz" "$out_dir" || rc=$?
	[[ $rc -eq 2 ]]
}

# Test 7: submission with extra files not in manifest (should still pass if diff matches ref)
_test_extra_files_not_in_manifest() {
	local tmp
	tmp=$(mktemp -d)
	trap 'rm -rf "$tmp"' RETURN
	local sub_dir="$tmp/sub"  ref_dir="$tmp/ref"  out_dir="$tmp/out"
	mkdir -p "$sub_dir" "$ref_dir"
	_copy_fixtures "$sub_dir"
	_copy_fixtures "$ref_dir"
	# Extra file in submission - manifest only lists the two standard files
	echo "extra data" > "$sub_dir/extra.txt"
	_make_manifest "$sub_dir"
	_make_manifest "$ref_dir"
	_make_submission_tarball "$tmp/submission.tar.gz" "$sub_dir"
	_make_submission_tarball "$tmp/reference.tar.gz"  "$ref_dir"
	# Extra file causes diff so exit 1
	local rc=0
	bash "$VERIFY" "$tmp/submission.tar.gz" "$tmp/reference.tar.gz" "$out_dir" || rc=$?
	[[ $rc -eq 1 ]]
}

# Test 8: submission with fewer files than manifest (SHA check fails - file missing)
_test_fewer_files_than_manifest() {
	local tmp
	tmp=$(mktemp -d)
	trap 'rm -rf "$tmp"' RETURN
	local sub_dir="$tmp/sub"  ref_dir="$tmp/ref"  out_dir="$tmp/out"
	mkdir -p "$sub_dir" "$ref_dir"
	# Only tally.json, no events.ndjson in submission
	cp "$FIXTURES/tally.json" "$sub_dir/tally.json"
	_copy_fixtures "$ref_dir"
	# Manifest still lists both files
	_make_manifest "$sub_dir"
	_make_manifest "$ref_dir"
	_make_submission_tarball "$tmp/submission.tar.gz" "$sub_dir"
	_make_submission_tarball "$tmp/reference.tar.gz"  "$ref_dir"
	local rc=0
	bash "$VERIFY" "$tmp/submission.tar.gz" "$tmp/reference.tar.gz" "$out_dir" || rc=$?
	# Missing file causes SHA check to fail (exit 3)
	[[ $rc -eq 3 ]]
}

# Test 9: report.md is created on success
_test_report_created_on_success() {
	local tmp
	tmp=$(mktemp -d)
	trap 'rm -rf "$tmp"' RETURN
	local sub_dir="$tmp/sub"  ref_dir="$tmp/ref"  out_dir="$tmp/out"
	mkdir -p "$sub_dir" "$ref_dir"
	_copy_fixtures "$sub_dir"
	_copy_fixtures "$ref_dir"
	_make_manifest "$sub_dir"
	_make_manifest "$ref_dir"
	_make_submission_tarball "$tmp/submission.tar.gz" "$sub_dir"
	_make_submission_tarball "$tmp/reference.tar.gz"  "$ref_dir"
	bash "$VERIFY" "$tmp/submission.tar.gz" "$tmp/reference.tar.gz" "$out_dir"
	[[ -f "$out_dir/report.md" ]]
}

# Test 10: report.md contains PASS on match
_test_report_contains_pass() {
	local tmp
	tmp=$(mktemp -d)
	trap 'rm -rf "$tmp"' RETURN
	local sub_dir="$tmp/sub"  ref_dir="$tmp/ref"  out_dir="$tmp/out"
	mkdir -p "$sub_dir" "$ref_dir"
	_copy_fixtures "$sub_dir"
	_copy_fixtures "$ref_dir"
	_make_manifest "$sub_dir"
	_make_manifest "$ref_dir"
	_make_submission_tarball "$tmp/submission.tar.gz" "$sub_dir"
	_make_submission_tarball "$tmp/reference.tar.gz"  "$ref_dir"
	bash "$VERIFY" "$tmp/submission.tar.gz" "$tmp/reference.tar.gz" "$out_dir"
	grep -q "PASS" "$out_dir/report.md"
}

# Test 11: report.md contains FAIL on mismatch
_test_report_contains_fail() {
	local tmp
	tmp=$(mktemp -d)
	trap 'rm -rf "$tmp"' RETURN
	local sub_dir="$tmp/sub"  ref_dir="$tmp/ref"  out_dir="$tmp/out"
	mkdir -p "$sub_dir" "$ref_dir"
	_copy_fixtures "$sub_dir"
	_copy_fixtures "$ref_dir"
	echo '{"stations":3,"total_votes":999}' > "$sub_dir/tally.json"
	local tampered_sha
	tampered_sha=$(sha256sum "$sub_dir/tally.json" | awk '{print $1}')
	_make_manifest "$sub_dir" "$tampered_sha" "$EVENTS_SHA256"
	_make_manifest "$ref_dir"
	_make_submission_tarball "$tmp/submission.tar.gz" "$sub_dir"
	_make_submission_tarball "$tmp/reference.tar.gz"  "$ref_dir"
	local rc=0
	bash "$VERIFY" "$tmp/submission.tar.gz" "$tmp/reference.tar.gz" "$out_dir" || rc=$?
	[[ $rc -eq 1 ]] && grep -q "FAIL" "$out_dir/report.md"
}

# Test 12: script creates output dir if it does not exist yet
_test_creates_output_dir() {
	local tmp
	tmp=$(mktemp -d)
	trap 'rm -rf "$tmp"' RETURN
	local sub_dir="$tmp/sub"  ref_dir="$tmp/ref"
	# output dir does not exist
	local out_dir="$tmp/new/nested/output"
	mkdir -p "$sub_dir" "$ref_dir"
	_copy_fixtures "$sub_dir"
	_copy_fixtures "$ref_dir"
	_make_manifest "$sub_dir"
	_make_manifest "$ref_dir"
	_make_submission_tarball "$tmp/submission.tar.gz" "$sub_dir"
	_make_submission_tarball "$tmp/reference.tar.gz"  "$ref_dir"
	bash "$VERIFY" "$tmp/submission.tar.gz" "$tmp/reference.tar.gz" "$out_dir"
	[[ -d "$out_dir" ]]
}

# ---------------------------------------------------------------------------
# Runner - plain bash harness with optional bats wrapper generation
# ---------------------------------------------------------------------------
# NOTE: @test syntax is bats-specific and breaks bash -n, so we drive tests
# via plain functions here. If bats is on PATH, export a .bats wrapper and
# delegate to it; otherwise run inline with the plain harness above.

_run_all_plain() {
	echo "Running tests (plain bash mode)"
	echo "verify-submission.sh: $VERIFY"
	echo ""
	_run_test "happy path: matching submission passes (exit 0)"            _test_happy_path
	_run_test "tampered tally.json causes diff (exit 1)"                   _test_tampered_tally
	_run_test "missing manifest.json rejected (exit 2)"                    _test_missing_manifest
	_run_test "wrong SHA-256 in manifest rejected (exit 3)"                _test_wrong_sha
	_run_test "empty tarball rejected (exit 2)"                            _test_empty_tarball
	_run_test "missing reference bundle rejected (exit 2)"                 _test_missing_reference
	_run_test "submission with extra files not in manifest (exit 1)"       _test_extra_files_not_in_manifest
	_run_test "submission with fewer files than manifest (exit 3)"         _test_fewer_files_than_manifest
	_run_test "report.md is created on success"                            _test_report_created_on_success
	_run_test "report.md contains PASS on match"                           _test_report_contains_pass
	_run_test "report.md contains FAIL on mismatch"                        _test_report_contains_fail
	_run_test "script creates output dir if it does not exist yet"         _test_creates_output_dir

	echo ""
	echo "Results: $_PASS passed, $_FAIL failed"
	if [[ ${#_FAILURES[@]} -gt 0 ]]; then
		echo "Failed tests:"
		for f in "${_FAILURES[@]}"; do
			echo "  - $f"
		done
		exit 1
	fi
}

if command -v bats &>/dev/null; then
	# Generate a temporary .bats file so we can use @test syntax without
	# embedding it in this file (which would break bash -n).
	BATS_TMP=$(mktemp /tmp/eel-verify-XXXXXX.bats)
	trap 'rm -f "$BATS_TMP"' EXIT
	cat > "$BATS_TMP" <<BATS_EOF
#!/usr/bin/env bats
load "${BASH_SOURCE[0]%/*}/../tests/test_verify.sh" 2>/dev/null || source "${BASH_SOURCE[0]}"

@test "happy path: matching submission passes (exit 0)" { _test_happy_path; }
@test "tampered tally.json causes diff (exit 1)" { _test_tampered_tally; }
@test "missing manifest.json rejected (exit 2)" { _test_missing_manifest; }
@test "wrong SHA-256 in manifest rejected (exit 3)" { _test_wrong_sha; }
@test "empty tarball rejected (exit 2)" { _test_empty_tarball; }
@test "missing reference bundle rejected (exit 2)" { _test_missing_reference; }
@test "submission with extra files not in manifest (exit 1)" { _test_extra_files_not_in_manifest; }
@test "submission with fewer files than manifest (exit 3)" { _test_fewer_files_than_manifest; }
@test "report.md is created on success" { _test_report_created_on_success; }
@test "report.md contains PASS on match" { _test_report_contains_pass; }
@test "report.md contains FAIL on mismatch" { _test_report_contains_fail; }
@test "script creates output dir if it does not exist yet" { _test_creates_output_dir; }
BATS_EOF
	export VERIFY FIXTURES TALLY_SHA256 EVENTS_SHA256
	export -f _make_submission_tarball _make_manifest _copy_fixtures
	export -f _test_happy_path _test_tampered_tally _test_missing_manifest _test_wrong_sha
	export -f _test_empty_tarball _test_missing_reference _test_extra_files_not_in_manifest
	export -f _test_fewer_files_than_manifest _test_report_created_on_success
	export -f _test_report_contains_pass _test_report_contains_fail _test_creates_output_dir
	bats "$BATS_TMP"
else
	_run_all_plain
fi
