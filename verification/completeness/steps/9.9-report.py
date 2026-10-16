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

"""
9.9-report.py: Render completeness-report.md from aggregated JSON and emit
completeness-attestation.json for the transparency log.

Reads:  completeness-report.json (written by run-audit.sh aggregator)
Writes: completeness-report.md, completeness-attestation.json
"""

import os
import sys
import json
import hashlib
import datetime
import urllib.request

REPO_ROOT = os.environ.get(
    "EEL_REPO_ROOT",
    os.path.abspath(os.path.join(os.path.dirname(__file__), "../../..")),
)
AUDIT_MODE = os.environ.get("EEL_AUDIT_MODE", "")
COMPLETENESS_DIR = os.path.join(REPO_ROOT, "verification/completeness")
REPORT_JSON = os.path.join(COMPLETENESS_DIR, "completeness-report.json")
REPORT_MD = os.path.join(COMPLETENESS_DIR, "completeness-report.md")
ATTEST_JSON = os.path.join(COMPLETENESS_DIR, "completeness-attestation.json")
TLOG_URL = os.environ.get("EEL_TLOG_URL", "http://localhost:3000/api/v1/log/append")


def log(msg: str) -> None:
    print(f"[9.9-report] {msg}", file=sys.stderr)


def render_markdown(report: dict) -> str:
    now = report.get("generated_at", datetime.datetime.utcnow().isoformat() + "Z")
    mode = report.get("audit_mode", "live")
    passed = report.get("passed", 0)
    failed = report.get("failed", 0)
    total = report.get("total_steps", 0)
    steps = report.get("steps", [])

    status = "PASS" if failed == 0 else "FAIL"
    lines = [
        "# Completeness Audit Report",
        "",
        f"Generated: {now}",
        f"Mode: {mode}",
        f"Result: **{status}** ({passed}/{total} steps passed)",
        "",
        "## Step Results",
        "",
        "| Step | Label | Status |",
        "|---|---|---|",
    ]
    for s in steps:
        icon = "PASS" if s.get("status") == "pass" else "FAIL"
        lines.append(f"| {s.get('step')} | {s.get('label')} | {icon} |")

    lines += [
        "",
        "## Summary",
        "",
        f"{passed} of {total} steps passed.",
    ]

    if failed == 0:
        lines.append("All completeness criteria met. Ready for Gate G6 sign-off.")
    else:
        failed_steps = [s.get("step") for s in steps if s.get("status") == "fail"]
        lines.append(
            f"Failing steps: {', '.join(failed_steps)}. Remediate before Gate G6."
        )

    lines += [
        "",
        "## Attestation",
        "",
        "After all steps pass, the report hash is submitted to the transparency log.",
        "The inclusion proof is stored in `completeness-attestation.json`.",
    ]

    return "\n".join(lines) + "\n"


def submit_to_tlog(report_bytes: bytes) -> dict:
    """Submit hash to transparency log; return inclusion proof or stub."""
    report_hash = hashlib.sha256(report_bytes).hexdigest()
    payload = json.dumps(
        {
            "kind": "meta-event",
            "subject": "completeness-audit",
            "sha256": report_hash,
            "timestamp": datetime.datetime.utcnow().isoformat() + "Z",
        }
    ).encode()

    try:
        req = urllib.request.Request(
            TLOG_URL,
            data=payload,
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        with urllib.request.urlopen(req, timeout=10) as resp:
            return json.loads(resp.read())
    except Exception as exc:
        log(f"WARN: tlog submission failed ({exc}); using stub proof")
        return {
            "sha256": report_hash,
            "inclusion_proof": None,
            "note": "tlog unavailable; submit manually",
        }


def main() -> int:
    if AUDIT_MODE == "dry":
        log("dry mode: generating stub report")
        report = {
            "version": "1",
            "generated_at": datetime.datetime.utcnow().isoformat() + "Z",
            "audit_mode": "dry",
            "total_steps": 9,
            "passed": 9,
            "failed": 0,
            "steps": [
                {"step": f"9.{i}", "label": f"step-{i}", "status": "pass"}
                for i in range(1, 10)
            ],
        }
    else:
        if not os.path.isfile(REPORT_JSON):
            log(f"FAIL: {REPORT_JSON} not found; run-audit.sh must complete first")
            return 1
        report = json.load(open(REPORT_JSON))

    # Render markdown
    md = render_markdown(report)
    with open(REPORT_MD, "w") as f:
        f.write(md)
    log(f"report written: {REPORT_MD}")

    # Attestation
    report_bytes = json.dumps(report, sort_keys=True).encode()
    if AUDIT_MODE == "dry":
        proof = {
            "sha256": hashlib.sha256(report_bytes).hexdigest(),
            "note": "dry mode stub",
        }
    else:
        proof = submit_to_tlog(report_bytes)

    attest = {
        "version": "1",
        "generated_at": datetime.datetime.utcnow().isoformat() + "Z",
        "report_sha256": hashlib.sha256(report_bytes).hexdigest(),
        "inclusion_proof": proof,
    }
    with open(ATTEST_JSON, "w") as f:
        json.dump(attest, f, indent=2)
    log(f"attestation written: {ATTEST_JSON}")

    if report.get("failed", 0) > 0:
        log(
            f"FAIL: {report['failed']} step(s) failed; report generated for remediation"
        )
        return 1

    log("PASS: completeness report and attestation generated")
    return 0


if __name__ == "__main__":
    sys.exit(main())
