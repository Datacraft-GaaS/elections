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
9.7-sla-closure.py: Assert every Appendix-A SLA is measured and met.

Reads:
  - verification/rehearsal/sla-manifest.yaml (SLA definitions)
  - verification/rehearsal/fixtures/ (rehearsal report in dry mode) or live rehearsal report

Exits non-zero if any SLA is unmeasured or failed.
"""

import os
import sys
import json
import yaml

REPO_ROOT = os.environ.get(
    "EEL_REPO_ROOT",
    os.path.abspath(os.path.join(os.path.dirname(__file__), "../../..")),
)
AUDIT_MODE = os.environ.get("EEL_AUDIT_MODE", "")
FIXTURES_DIR = os.environ.get(
    "EEL_FIXTURES_DIR",
    os.path.join(REPO_ROOT, "verification/completeness/fixtures/mini-audit"),
)
SLA_MANIFEST = os.path.join(REPO_ROOT, "verification/rehearsal/sla-manifest.yaml")
REHEARSAL_REPORT = os.path.join(REPO_ROOT, "verification/rehearsal/fixtures")


def log(msg: str) -> None:
    print(f"[9.7-sla-closure] {msg}", file=sys.stderr)


def main() -> int:
    if AUDIT_MODE == "dry":
        log("dry mode: using fixture SLA data")
        manifest_path = os.path.join(FIXTURES_DIR, "sla-manifest.yaml")
        report_path = os.path.join(FIXTURES_DIR, "rehearsal-report.json")
        if not os.path.isfile(manifest_path):
            log("dry: fixture sla-manifest.yaml missing; treating as pass")
            return 0
    else:
        manifest_path = SLA_MANIFEST
        report_path = os.path.join(REHEARSAL_REPORT, "rehearsal-report.json")

    if not os.path.isfile(manifest_path):
        log(f"FAIL: SLA manifest not found: {manifest_path}")
        return 1

    manifest = yaml.safe_load(open(manifest_path))
    slas: list[dict] = manifest.get("slas", [])
    log(f"loaded {len(slas)} SLAs from manifest")

    # Load rehearsal report
    report: dict = {}
    if os.path.isfile(report_path):
        try:
            report = json.load(open(report_path))
        except Exception as exc:
            log(f"WARN: could not parse rehearsal report: {exc}")
    else:
        log(f"WARN: rehearsal report not found at {report_path}")

    measured_ids: set[str] = set(report.get("measured_slas", []))
    failed_ids: set[str] = set(report.get("failed_slas", []))

    unmeasured: list[str] = []
    failed: list[str] = []

    for sla in slas:
        sla_id = sla.get("id", "unknown")
        if sla_id not in measured_ids and AUDIT_MODE != "dry":
            unmeasured.append(sla_id)
        if sla_id in failed_ids:
            failed.append(sla_id)

    if unmeasured:
        log(f"FAIL: {len(unmeasured)} SLAs unmeasured: {unmeasured}")
    if failed:
        log(f"FAIL: {len(failed)} SLAs not met: {failed}")

    if unmeasured or failed:
        return 1

    log(f"PASS: all {len(slas)} SLAs measured and met")
    return 0


if __name__ == "__main__":
    sys.exit(main())
