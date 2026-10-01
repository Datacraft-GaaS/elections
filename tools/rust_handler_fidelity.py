#!/usr/bin/env python3
"""Measure how faithfully each service handler implements its own OpenAPI contract.

Every contract in packages/openapi-contracts/ declares `x-handler-bound`, which
names the Rust function that serves the operation. That binding is what makes
ADR-0011 checkable rather than aspirational: a contract says a response exists, a
handler claims it, and a reader can go look. It also means divergence between
the two is mechanically detectable, which is what this tool measures.

A handler is FABRICATED when it returns a success response it did not earn --
no store, no client, no query, nothing persisted -- so the response describes a
world where the work happened. The rts-svc get_form_proof is the clearest case:
the contract declares 404 for an unknown form, and the handler returns 200 with
an empty proof and a Merkle root of 64 zeros, for every form id, forever.

This is a reporting tool, not a gate. A FABRICATED handler is not automatically
wrong: a service deliberately stubbed for a demo or an integration test is a
legitimate choice, and only a human knows which. What the tool refuses to do is
let the count live in someone's head. Read the list, decide per service, and
write the decision into status.md.

Usage:
    python3 tools/rust_handler_fidelity.py            # summary per service
    python3 tools/rust_handler_fidelity.py --services rts-svc
    python3 tools/rust_handler_fidelity.py --verbose  # every finding, with body
"""

from __future__ import annotations

import argparse
import re
import sys
from collections import defaultdict
from pathlib import Path

import tomllib

import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "packages" / "openapi-contracts"

# Reaching for one of these means the answer came from somewhere durable: a
# store, a database, or a downstream service.
WORK_MARKERS = re.compile(
    r"\b(repo|repos|store|storage|db|pool|client|svc|service|dao|"
    r"query|fetch_|insert|upsert|delete_from|execute|commit|"
    r"sqlx|rusqlite|diesel|repository)\b",
    re.I,
)

# Reading or mutating in-process state. A handler that touches a shared map,
# mutex or store is answering from real data even with no database behind it --
# `acknowledge_alert` locks `state.alerts`, mutates the record, and 404s when
# the id is unknown, which is about as honest as a handler gets. The first
# version of this tool called it FABRICATED because its body was short.
STATE_MARKERS = re.compile(
    r"\bstate\s*\.|\bstore\s*\.|\bguard\b|\.lock\(\)|\.try_lock\(\)|"
    r"\.read\(\)|\.write\(\)|get_mut|\.insert\(|\.remove\(|\.entry\(|"
    r"\.extend\(|\.retain\(|\.get\(&|\.get_mut\(",
)

# A handler that can fail is one that consulted something. Returning 404 on a
# missing key or propagating Err is evidence of a lookup, and it is the
# cheapest reliable signal that a response was derived rather than invented.
ERROR_BRANCH = re.compile(
    r"StatusCode::(?:NOT_FOUND|BAD_REQUEST|UNAUTHORIZED|FORBIDDEN|CONFLICT|"
    r"UNPROCESSABLE_ENTITY|SERVICE_UNAVAILABLE|INTERNAL_SERVER_ERROR|"
    r"TOO_MANY_REQUESTS|NOT_ACCEPTABLE)\b"
    r"|\bNone\s*=>|\bErr\(|\.is_err\(\)|\bSome\("
)

# A handler body that is a single expression and short enough to read in one
# glance is a candidate; length alone does not decide it, WORK_MARKERS does.
# Signature included, so this is a deliberately generous ceiling: a handler
# longer than this is doing enough that its verdict is not worth guessing at.
FABRICATED_MAX_LINES = 25


def contract_operations() -> list[dict]:
    """Every operation in every contract, with its declared responses."""
    out = []
    for spec in sorted(CONTRACTS.glob("*/openapi.yaml")):
        try:
            doc = yaml.safe_load(spec.read_text()) or {}
        except Exception as exc:  # a malformed contract is itself a finding
            print(f"  contract parse error: {spec.relative_to(ROOT)}: {exc}",
                  file=sys.stderr)
            continue
        svc = spec.parent.name
        for path, methods in (doc.get("paths") or {}).items():
            for verb, op in (methods or {}).items():
                if not isinstance(op, dict):
                    continue
                out.append({
                    "service": svc,
                    "contract": spec.relative_to(ROOT),
                    "path": path,
                    "verb": verb,
                    "operation_id": op.get("operationId"),
                    "bound": op.get("x-handler-bound") or "",
                    "handler": (op.get("x-handler-bound") or "").split(".")[-1],
                    "responses": sorted((op.get("responses") or {}).keys()),
                })
    return out


def crate_names() -> dict[str, str]:
    """Map each crate directory to the package name in its Cargo.toml.

    `x-handler-bound` names the crate as declared in Cargo.toml, which is not
    the directory name: `bvr-svc.cvr.list_events` refers to a function inside
    the crate whose package is `cvr`, and its sources may live in a directory
    called `handlers`. Resolving against directory names answers the wrong
    crate, so Cargo.toml is the authority.
    """
    names: dict[str, str] = {}
    for manifest in (ROOT / "services").rglob("Cargo.toml"):
        if "/target/" in str(manifest):
            continue
        try:
            doc = tomllib.loads(manifest.read_text())
        except Exception:
            continue
        pkg = doc.get("package", {})
        if not isinstance(pkg, dict) or "name" not in pkg:
            continue
        names[str(manifest.parent.resolve())] = pkg["name"]
    return names


def handler_bodies() -> dict[str, dict]:
    """Index every `pub async fn` by crate and by bare name.

    The crate key matters: `x-handler-bound` is `<service>-svc.<crate>.<fn>`,
    and bare function names collide across services -- `list_snapshots` exists
    in both rts-svc and crms-api. Indexing by name alone silently answers the
    cvr contract with rts-svc's body, which is a wrong measurement that looks
    like a finding, so the crate is part of the key and the name lookup is
    only ever a fallback for contracts whose binding omits it.
    """
    bodies: dict[str, dict] = {}
    crates = crate_names()
    for rs in sorted((ROOT / "services").rglob("*.rs")):
        if "/target/" in str(rs):
            continue
        try:
            text = rs.read_text(errors="replace")
        except Exception:
            continue
        # Not just `pub fn`: handlers in route modules are routinely private
        # (`async fn list_events(...)`), and indexing only public ones reports
        # a fully implemented handler as MISSING.
        for match in re.finditer(r"^(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+(\w+)",
                                 text, re.M):
            name = match.group(1)
            crate = _crate_of(rs, crates)
            body = _extract_body(text, match.end())
            if body is None:
                continue
            entry = {"file": rs.relative_to(ROOT), "body": body,
                     "crate": crate, "fn": name}
            # A bare name is ambiguous; only the first claimant gets it.
            bodies.setdefault(name, entry)
            if crate:
                bodies[f"{crate}::{name}"] = entry
    return bodies


def _crate_of(path: Path, crates: dict[str, str]) -> str:
    """The Cargo.toml package name of the crate a source file belongs to."""
    for parent in path.parents:
        if str(parent.resolve()) in crates:
            return crates[str(parent.resolve())]
    return ""


def service_dir(segment: str) -> Path | None:
    """`bvi-svc` -> services/bvi-svc, the directory the binding's first part names."""
    if not segment:
        return None
    for candidate in (ROOT / "services" / segment,
                      ROOT / "services" / f"{segment}-svc"):
        if candidate.is_dir():
            return candidate
    return None


def resolve(op: dict, bodies: dict[str, dict]) -> dict | None:
    """Find the handler this contract names, crate-scoped, or nothing.

    There is deliberately no fallback to a bare function name once the binding
    names a crate. An earlier version of this tool fell back, and answered the
    cvr contract's `bvr-svc.cvr.list_events` with `list_events` out of
    crms-api -- a different service, a different meaning, reported as if it were
    the bound handler. A gap is the correct answer when the named handler is
    absent; inventing a neighbour is not.
    """
    parts = (op.get("bound") or "").split(".")
    if len(parts) < 3:
        matches = [e for key, e in bodies.items() if key == op["handler"]]
        return matches[0] if len(matches) == 1 else None

    service, crate, fn = parts[0], parts[-2], parts[-1]
    scoped = bodies.get(f"{crate}::{fn}")
    if scoped is not None:
        return scoped

    # The crate segment is often aspirational -- `bvi-svc.bvi.*` names a crate
    # called `bvi` that does not exist. Fall back to a unique match inside the
    # service the binding names, which is the part that is reliable. Uniqueness
    # matters: two candidates means the binding is genuinely ambiguous and the
    # honest answer is that it resolves to nothing.
    root = service_dir(service)
    if root is None:
        return None
    matches = [e for e in bodies.values()
               if e["fn"] == fn and str(root.resolve()) in str(e["file"].resolve())]
    unique = {str(e["file"]): e for e in matches}
    return next(iter(unique.values())) if len(unique) == 1 else None


def _extract_body(text: str, start: int) -> str | None:
    """Grab a whole function by brace matching: signature *and* body.

    The signature is not optional. `Json(_req): Json<Request>` lives there, and
    it is the entire evidence that a handler discards its input. An earlier
    version started at the opening brace and returned only the body, so the
    discarded-extractor signal was never in the text being searched and
    IGNORES_INPUT silently never fired.
    """
    brace = text.find("{", start)
    if brace == -1:
        return None
    depth = 0
    for i in range(brace, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return text[start : i + 1]
    return None


# A health or readiness probe answering `{"ok": true}` is telling the truth: the
# process is up. Judging it as fabricated would bury the real findings under
# noise, which is how a real finding stays unread.
PROBE_OPERATION = re.compile(r"healthz|health|readyz|readiness|liveness|ping", re.I)

# `Json(_req)`, `State(_state)`: the leading underscore is how a stub silences
# the unused-variable warning, so the handler compiles clean while discarding
# the request it was given. A handler that ignores its own input cannot be
# honest about the answer, whatever it returns.
IGNORED_EXTRACTOR = re.compile(
    r"\b(Json|Query|Path|State|Form|Extension)\(\s*_\w+", re.M
)

# A handler that says out loud that it has no implementation is honest, and
# must be counted separately. Without this the fabricated total can never fall,
# which is how a metric stops being a signal: a number that cannot improve is
# decoration.
DECLARES_UNIMPLEMENTED = re.compile(
    r"NotImplemented|NOT_IMPLEMENTED|not_implemented", re.M
)

# An idempotency record should hold the response it stands in for. Storing an
# empty object and then returning a claimed result means the first request gets
# the claim and every repeat gets `{}` -- the same call answered two different
# ways depending on whether it had been made before. It is mechanically
# detectable, which matters: 13 handlers in mobile-bff carried it, and a bug
# that changes the answer on retry is the kind that survives review of the
# happy path.
EMPTY_IDEMPOTENCY_RECORD = re.compile(
    r"store\(\s*&\w*idempotency\w*\s*,\s*&serde_json::json!\(\{\}\)\s*\)"
)


def classify(body: str, op: dict | None = None) -> str:
    """VERIFIED, DECLARES_UNIMPLEMENTED, EMPTY_IDEMPOTENCY, IGNORES_INPUT,
    FABRICATED, UNKNOWN.

    DECLARES_UNIMPLEMENTED is checked first so that a handler converted to an
    honest 501 stops being counted as fabricated, even though its signature
    still carries the underscore-prefixed extractors it never used.
    """
    lines = [ln for ln in body.splitlines() if ln.strip() and not ln.strip().startswith("//")]
    if not lines:
        return "FABRICATED"
    if DECLARES_UNIMPLEMENTED.search(body):
        return "DECLARES_UNIMPLEMENTED"
    if IGNORED_EXTRACTOR.search(body):
        return "IGNORES_INPUT"
    if EMPTY_IDEMPOTENCY_RECORD.search(body):
        return "EMPTY_IDEMPOTENCY"
    if op and PROBE_OPERATION.search(op.get("operation_id") or ""):
        return "VERIFIED"
    if op and PROBE_OPERATION.search(op.get("path") or ""):
        return "VERIFIED"
    # Three independent ways to have earned the answer. Missing all three while
    # returning only a constructed success is what fabrication actually looks
    # like, and it is what the earlier heuristic kept getting wrong.
    if WORK_MARKERS.search(body) or STATE_MARKERS.search(body) \
            or ERROR_BRANCH.search(body):
        return "VERIFIED"
    if len(lines) <= FABRICATED_MAX_LINES:
        return "FABRICATED"
    return "UNKNOWN"


METHOD_CALL = re.compile(r"\b(get|post|put|patch|delete)\(\s*(\w+)\s*\)")


def route_registrations(text: str) -> list[tuple[str, str]]:
    """Every (path, handler) pair registered by an axum `.route(...)` call.

    Contract fidelity only covers operations a contract names. An axum router
    can serve any handler on any path, so a handler with no binding is invisible
    to that measurement -- and the three worst fabrications found so far
    (voter lookup, tally verify, certificate verify) were all invisible for
    exactly that reason. This enumerates registrations directly, so an unbound
    fabrication shows up as itself rather than as an absence.

    Handles `.route("/p", get(h))`, the method-chained
    `.route("/p", get(a).post(b))`, and paths wrapped across lines.
    """
    out: list[tuple[str, str]] = []
    for m in re.finditer(r"\.route\s*\(", text):
        block = _balanced(text, m.end() - 1)
        if block is None:
            continue
        pm = re.search(r'"([^"]*)"', block)
        path = pm.group(1) if pm else "?"
        for _verb, handler in METHOD_CALL.findall(block):
            out.append((path, handler))
    return out


def _balanced(text: str, open_at: int) -> str | None:
    """The parenthesised group starting at `open_at`, matched by depth."""
    depth = 0
    for i in range(open_at, len(text)):
        if text[i] == "(":
            depth += 1
        elif text[i] == ")":
            depth -= 1
            if depth == 0:
                return text[open_at + 1 : i]
    return None


def all_routes(bodies: dict[str, dict]) -> dict[str, list[str]]:
    """handler name -> the paths it is registered on, across the services tree."""
    routes: dict[str, list[str]] = {}
    for rs in sorted((ROOT / "services").rglob("*.rs")):
        if "/target/" in str(rs):
            continue
        try:
            text = rs.read_text(errors="replace")
        except Exception:
            continue
        for path, handler in route_registrations(text):
            routes.setdefault(handler, []).append(path)
    return routes


def report_unbound(ops: list[dict], bodies: dict[str, dict]) -> int:
    """Fabrications among registered handlers that no contract binds.

    This is the measurement the bound-only view cannot give. A contract that
    names a handler is the easy case; a handler on a live route with no binding
    is the one that was invisible when verify_voter_lookup answered
    `verified: true` for an invented voter.
    """
    bound = {op["handler"] for op in ops if op["handler"]}
    routes = all_routes(bodies)

    total = fab = 0
    rows: list[tuple[str, str, str, str]] = []
    for handler, paths in sorted(routes.items()):
        entry = bodies.get(handler)
        if entry is None:
            continue
        total += 1
        verdict = classify(entry["body"], None)
        if verdict in ("VERIFIED", "DECLARES_UNIMPLEMENTED", "UNKNOWN"):
            continue
        fab += 1
        rows.append((verdict, handler, str(entry["file"]), paths[0]))

    unbound = [r for r in rows if r[1] not in bound]
    print(f"registered handlers judged : {total}")
    print(f"fabricated                 : {fab}")
    print(f"  of which contract-unbound: {len(unbound)}")
    print(f"  contract-bound           : {len(rows) - len(unbound)}")
    print()
    if not unbound:
        print("no unregistered fabrications")
        return 0
    print("contract-unbound fabrications (invisible to the bound-only view)")
    print("-" * 78)
    for verdict, handler, f, path in unbound:
        print(f"[{verdict:<15}] {handler:<28} {path}")
        print(f"                  {f}")
    return 0


# Hand-checked against the source, one verdict each. The classifier has been
# wrong in ways that looked confident -- it called `acknowledge_alert`
# fabricated because the body was short, and resolved contract bindings by bare
# function name so it answered one service with another's handler. A heuristic
# with no test is just a guess that prints a number.
#
# This has already earned its keep: converting certify_county_result and
# amend_form34a to 501s dropped this from 7/7 to 5/7, because the expectations
# recorded the old behaviour. A test that cannot fail is worse than no test.
SELF_TEST = [
    # (handler, file, expected) -- verified by reading the handler.
    ("acknowledge_alert", "services/support-admin-svc/crates/intent/src/http.rs",
     "VERIFIED"),   # locks state.alerts, mutates, 404s on unknown id
    ("verify_voter_lookup", "services/field-mobile-svc/crates/mobile-bff/src/spec.rs",
     "DECLARES_UNIMPLEMENTED"),
    ("tally_verify", "services/field-mobile-svc/crates/mobile-bff/src/spec.rs",
     "DECLARES_UNIMPLEMENTED"),
    ("get_reports", "services/abis-svc/crates/abis-api/src/routes/reports.rs",
     "DECLARES_UNIMPLEMENTED"),
    # Converted to honest 501s; previously logged and returned 204 while
    # certifying nothing, and logged then returned 202 with a minted
    # amendment_id while discarding the amendment.
    ("certify_county_result", "services/governance-svc/crates/referendum/src/handlers.rs",
     "DECLARES_UNIMPLEMENTED"),
    ("amend_form34a", "services/rts-svc/crates/rts-ro-api/src/handlers.rs",
     "DECLARES_UNIMPLEMENTED"),
    ("declare_referendum", "services/governance-svc/crates/referendum/src/handlers.rs",
     "DECLARES_UNIMPLEMENTED"),
    ("get_national_result", "services/governance-svc/crates/referendum/src/handlers.rs",
     "DECLARES_UNIMPLEMENTED"),
    ("assign_adjudication", "services/abis-svc/crates/abis-api/src/routes/adjudication.rs",
     "IGNORES_INPUT"),  # State(_state) unused, echoes req as "in_review"
]


def self_test() -> int:
    bodies = handler_bodies()
    bad = 0
    for handler, path, expected in SELF_TEST:
        entry = bodies.get(handler)
        if entry is None:
            print(f"MISSING  {handler} (not indexed)")
            bad += 1
            continue
        got = classify(entry["body"], None)
        want = expected
        ok = got == want
        bad += 0 if ok else 1
        mark = "ok  " if ok else "FAIL"
        print(f"{mark} {handler:<24} want={want:<24} got={got}")
        if not ok and str(entry["file"]) != path:
            print(f"       note: resolved in {entry['file']}, expected {path}")
    print(f"\n{len(SELF_TEST) - bad}/{len(SELF_TEST)} classifier checks pass")
    return 1 if bad else 0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--services", help="comma-separated service filter")
    ap.add_argument("--verbose", action="store_true", help="print every finding")
    ap.add_argument(
        "--self-test",
        action="store_true",
        help="check the classifier against hand-verified handlers",
    )
    ap.add_argument(
        "--all-routes",
        action="store_true",
        help="judge every registered axum handler, including ones no contract binds",
    )
    args = ap.parse_args()

    if args.self_test:
        return self_test()

    ops = contract_operations()
    bodies = handler_bodies()

    if args.all_routes:
        return report_unbound(ops, bodies)

    wanted = {s.strip() for s in args.services.split(",")} if args.services else None

    per_service: dict[str, dict] = defaultdict(
        lambda: {"declared": 0, "fabricated": 0, "ignores_input": 0,
                 "declared_unimpl": 0, "empty_idem": 0, "missing": 0, "unknown": 0}
    )
    findings = []

    for op in ops:
        svc = op["service"]
        if wanted and svc not in wanted:
            continue
        # Only operations that name a handler can be judged; the rest are
        # contract-level work, out of scope here.
        if not op["handler"]:
            continue
        per_service[svc]["declared"] += 1
        found = resolve(op, bodies)
        if found is None:
            per_service[svc]["missing"] += 1
            findings.append((svc, op, None, "MISSING"))
            continue
        verdict = classify(found["body"], op)
        if EMPTY_IDEMPOTENCY_RECORD.search(found["body"]):
            per_service[svc]["empty_idem"] += 1
        if verdict in ("FABRICATED", "IGNORES_INPUT"):
            per_service[svc]["fabricated"] += 1
            if verdict == "IGNORES_INPUT":
                per_service[svc]["ignores_input"] += 1
        elif verdict == "DECLARES_UNIMPLEMENTED":
            per_service[svc]["declared_unimpl"] += 1
        elif verdict == "EMPTY_IDEMPOTENCY":
            per_service[svc]["fabricated"] += 1
        elif verdict == "UNKNOWN":
            per_service[svc]["unknown"] += 1
        if verdict != "VERIFIED":
            findings.append((svc, op, found, verdict))

    print(f"{'service':<20} {'bound':>6} {'fabricated':>11} {'ignores':>8} "
          f"{'honest501':>9} {'emptyIdem':>10} {'missing':>8} {'unknown':>8}")
    print("-" * 86)
    total_fab = 0
    for svc in sorted(per_service):
        c = per_service[svc]
        total_fab += c["fabricated"]
        print(f"{svc:<20} {c['declared']:>6} {c['fabricated']:>11} "
              f"{c['ignores_input']:>8} {c['declared_unimpl']:>9} "
              f"{c['empty_idem']:>10} {c['missing']:>8} {c['unknown']:>8}")
    print("-" * 86)
    print(f"{'TOTAL':<20} {'':>6} {total_fab:>11}")

    if args.verbose:
        print("\nfindings")
        for svc, op, found, verdict in findings:
            print(f"\n[{verdict}] {svc} {op['verb'].upper()} {op['path']}")
            print(f"  handler: {op['handler']}  (bound: {op['bound']})")
            if found:
                print(f"  file: {found['file']}")
                for ln in found["body"].strip().splitlines()[:6]:
                    print(f"    {ln.strip()}")
    else:
        print(f"\n{total_fab} fabricated handler(s); "
              f"re-run with --verbose, or --services <name>")
    return 0


if __name__ == "__main__":
    sys.exit(main())
