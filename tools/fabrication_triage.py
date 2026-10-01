#!/usr/bin/env python3
"""Classify each fabricated handler as implementable here, or blocked on something absent.

The fidelity scanner answers "does this handler fabricate a response". It does not
answer "can this be fixed in this workspace", and treating those as the same
question is how a backlog of 238 becomes an unactionable number. Most of these
handlers genuinely cannot be completed here: they need the M02 BVR snapshot API,
an OCR engine, an HSM, a transactional outbox, Kafka, or a gazetteer, none of
which exist in this repository. Converting them to 501 would be honest but would
also remove capability that a 501 cannot replace, and leaving them fabricated is
worse than either.

This sorts every fabrication into one of two buckets and prints the counts:

  IMPLEMENTABLE   the work is local: create/read/update against AppState, or a
                  computation over inputs the request already carries. These are
                  worth doing first and eel-memstore-rs is usually enough.
  NEEDS_BUILD     the response carries a cryptographic artifact (a Merkle root, an
                  inclusion proof, a signature, a build fingerprint) or asserts a
                  verdict as a literal `true`. Neither can come from a request
                  body or a HashMap: they need an append-only log, a key, or the
                  attestation service that would actually perform the check. These
                  are subsystems to build, not stores to add.
  BLOCKED         the handler's body or its doc comment names an external system
                  that does not exist here. The named system is recorded, so the
                  blocker is a specific thing to build rather than a shrug.

A handler counts as BLOCKED when its body *or its doc comment* names one of the
missing systems. Comments are deliberately included: several of these handlers
document their own blocker ("no adapter crate exists in this workspace"), which
is the most reliable evidence available and was previously being thrown away.

Usage:
    python3 tools/fabrication_triage.py                 # counts per crate
    python3 tools/fabrication_triage.py --list          # every finding
    python3 tools/fabrication_triage.py --crate ecc-api # one crate in detail
    python3 tools/fabrication_triage.py --markdown      # for pasting into status.md
"""

from __future__ import annotations

import argparse
import importlib.util
import re
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# External systems named by these handlers that do not exist in this workspace.
# Keyed by a lowercase token; the value is what would have to be built.
EXTERNAL = {
    "m02": "M02 BVR snapshot adapter",
    "bvr": "BVR snapshot client (M02)",
    "ocr": "OCR engine",
    "hsm": "HSM / signature verification",
    "outbox": "transactional outbox",
    "kafka": "Kafka broker",
    "spicedb": "SpiceDB authorization",
    "gazetteer": "station gazetteer",
    "geocod": "geocoding service",
    "s3": "object storage",
    "m50": "M50 by-election trigger",
    "m04": "M04 register subset service",
    "valkey": "Valkey cache",
    "fred": "Valkey cache (fred client)",
    "minio": "object storage (MinIO)",
}

# Responses containing one of these are not local computations. A Merkle root, an
# inclusion proof or a signature cannot be produced from a request body or a
# HashMap: they require an append-only log or a key, which is a subsystem to
# build, not a store to drop in. Treating `root_hash: "0".repeat(64)` as
# "implementable with memstore" would be the same kind of optimism that produced
# it -- an empty transparency log and a real one are indistinguishable from the
# response alone.
# A verdict asserted as a literal `true` with nothing behind it. These are not
# local computations either: asserting that a build fingerprint passed, or that a
# clone check is valid, requires the attestation service or HSM that would
# actually perform the check. mobile-bff's binary_integrity_check returns
# passed: true for any build_fingerprint and any attestation_token, which is an
# integrity gate wired to always open.
ASSERTED_VERDICT = re.compile(
    r"\b(passed|is_valid|verified|is_clone|is_genuine|match_score|"
    r"signature_valid|certificate_valid)\s*:\s*true\b",
    re.I,
)

CRYPTO_ARTIFACT = re.compile(
    r"merkle_root|root_hash|merkle_proof|inclusion_proof|\bsignature\b|"
    r"proof_hash|attestation_hash|chain_of_custody_hash",
    re.I,
)


def load_fidelity_module():
    spec = importlib.util.spec_from_file_location(
        "fidelity", ROOT / "tools" / "rust_handler_fidelity.py"
    )
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def classify_blockers(text: str) -> list[str]:
    """Named external systems referenced in a handler's body or its doc comment."""
    low = text.lower()
    hits = []
    for token, name in EXTERNAL.items():
        # Word-ish match, tolerant of M02/M-02 and bvr_store style spellings.
        if re.search(rf"\b{token}[-_]?\b", low):
            hits.append(name)
    return hits


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--list", action="store_true", help="print every finding")
    ap.add_argument("--crate", help="restrict to one crate path fragment")
    ap.add_argument("--markdown", action="store_true", help="emit a status.md table")
    args = ap.parse_args()

    f = load_fidelity_module()
    ops = f.contract_operations()
    bodies = f.handler_bodies()
    routes = f.all_routes(bodies)
    bound = {op["handler"] for op in ops if op["handler"]}

    impl: list[tuple[str, str, str]] = []      # handler, file, route
    needs_build: list[tuple[str, str, str, str]] = []
    blocked: list[tuple[str, str, str, list[str]]] = []

    for handler, paths in routes.items():
        entry = bodies.get(handler)
        if entry is None:
            continue
        verdict = f.classify(entry["body"], {"operation_id": "", "path": paths[0]})
        if verdict in ("VERIFIED", "DECLARES_UNIMPLEMENTED", "UNKNOWN"):
            continue
        file = str(entry["file"])
        if args.crate and args.crate not in file:
            continue
        blockers = classify_blockers(entry["body"])
        artifact = CRYPTO_ARTIFACT.search(entry["body"])
        verdict = ASSERTED_VERDICT.search(entry["body"])
        if blockers:
            blocked.append((handler, file, paths[0], blockers))
        elif artifact:
            needs_build.append((handler, file, paths[0], artifact.group(0)))
        elif verdict:
            needs_build.append(
                (handler, file, paths[0], f"{verdict.group(1)}: true")
            )
        else:
            impl.append((handler, file, paths[0]))

    if args.markdown:
        per_crate = defaultdict(lambda: {"impl": 0, "build": 0, "blocked": 0})
        for h, file, _ in impl:
            per_crate[file]["impl"] += 1
        for h, file, _, _a in needs_build:
            per_crate[file]["build"] += 1
        for h, file, _, _b in blocked:
            per_crate[file]["blocked"] += 1
        print("| crate | implementable | needs a log or key | blocked on absent system |")
        print("|---|---|---|---|")
        for file in sorted(per_crate):
            c = per_crate[file]
            short = file.replace("services/", "")
            print(f"| `{short}` | {c['impl']} | {c['build']} | {c['blocked']} |")
        ti = sum(c["impl"] for c in per_crate.values())
        tb = sum(c["build"] for c in per_crate.values())
        tc = sum(c["blocked"] for c in per_crate.values())
        print(f"| **total** | **{ti}** | **{tb}** | **{tc}** |")
        return 0

    per_crate = defaultdict(lambda: {"impl": 0, "build": 0, "blocked": 0})
    for h, file, _ in impl:
        per_crate[file]["impl"] += 1
    for h, file, _, _a in needs_build:
        per_crate[file]["build"] += 1
    for h, file, _, _b in blocked:
        per_crate[file]["blocked"] += 1

    print(f"{'crate':<50} {'implementable':>13} {'needs build':>11} {'blocked':>9}")
    print("-" * 86)
    for file in sorted(per_crate, key=lambda k: -(per_crate[k]["impl"])):
        c = per_crate[file]
        print(f"{file.replace('services/', ''):<50} {c['impl']:>13} {c['build']:>11} {c['blocked']:>9}")
    print("-" * 86)
    print(f"{'TOTAL':<50} {len(impl):>13} {len(needs_build):>11} {len(blocked):>9}")

    if args.list:
        print("\nBLOCKED -- needs something that does not exist here")
        for handler, file, route, blockers in sorted(blocked):
            print(f"  {handler:<34} {', '.join(blockers)}")
        print("\nNEEDS BUILD -- response is a crypto artifact, needs a log or a key")
        for handler, file, route, artifact in sorted(needs_build):
            print(f"  {handler:<34} {route}  [{artifact}]")
        print("\nIMPLEMENTABLE -- local work, memstore is usually enough")
        for handler, file, route in sorted(impl):
            print(f"  {handler:<34} {route}")
    return 0


if __name__ == "__main__":
    sys.exit(main())