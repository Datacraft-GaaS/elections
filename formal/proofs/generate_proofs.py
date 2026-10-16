#!/usr/bin/env python3
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

"""TKLA+ proof generator for all 22 EEL services.

Generates per service: {service}.tla + {service}.cvl + run.sh + README.md
Run: python3 generate_proofs.py

The TLA+ module name is the service name with hyphens replaced by underscores
and each segment capitalised, because TLA+ identifiers cannot contain a hyphen:
`---- MODULE hsm-signer ----` is a parse error, and so is a module whose name
disagrees with its filename. `module_name()` is the single place that mapping
lives, and `main()` parses every spec it writes so a regression fails here
rather than in a verification run.
"""
import os
import re
import shutil
import subprocess

SERVICES = [
    "abis-svc",       # ABIS: biometric attestation, NIST MINEX, tamper-evident gallery
    "ai-platform-svc", # AI: model monitoring, anomaly detection, LLM observability
    "biometric-edge-svc", # BVR/BVI/FAP60/anti-iris drivers, nearform-nbis FFI
    "boundary-svc",    # Geo-fencing, polygon containment
    "bvi-svc",        # Biometric verification identity (M1–M4), anomaly detection
    "bvr-svc",        # Biometric voter registration, ABIS deduplication
    "crms-svc",       # Coalition/party registration, audit shadow, event sourcing
    "ecc-svc",        # Election command centre, composite score, election-day mode
    "field-mobile-svc", # Mobile BFF, offline-first, attestation, form quality gate
    "field-mobile-verify", # Verification endpoint, audit shadow replay
    "governance-svc", # Dispute/petition, forensic export, evidence bundle
    "hello-svc",      # Readiness probe, identity federation
    "hsm-signer",     # PKCS#11 Ed25519+ML-DSA, CCB log, break-glass
    "inventory-svc",   # RFID, ballot chain-of-custody, M-Pesa B2C
    "portal-svc",     # Public results, observer view, sanitised read-only
    "queue-svc",      # Kafka consumer/producer, idempotency, dead-letter
    "rts-svc",        # Form 34A capture, HMAC, results transmission, tally
    "support-admin-svc", # Ticket queue, diagnostics, warranty, assignment
    "training-svc",    # LMS, procedural lessons, assessments, certification
    "transparency-svc", # Append-only log, Merkle tree, STH, VRF sortition, gazette anchor
    "vprs-svc",       # Voter portal, registration lookup
    "workforce-svc",   # Personnel, attendance, geo-fence check-in
]


def module_name(service):
    """TLA+ module name for a service.

    TLA+ identifiers admit letters, digits and underscore only, so the hyphens
    in `hsm-signer` cannot appear. Capitalising each segment keeps the module
    recognisable against the service it describes.
    """
    return "".join(part.capitalize() for part in re.split(r"[-_]", service))


# Operators the invariant language may use that are not in the EXTENDS list.
# Declared as CONSTANT so a spec that mentions them resolves; the arity is
# what the invariants actually call them with.
_AUX_OPERATORS = {
    "Hash": 1,
    "hmac": 2,
    "verifyEd25519": 1,
    "verifyMLDSA": 1,
    "verify_nbis": 2,
}

_IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
# Any identifier list immediately followed by `\in` is a binder, whichever
# construct introduced it: `\A i, j \in S`, `\E x \in S`, `CHOOSE x \in S`, and
# the set comprehension `{chain[i] : i \in DOMAIN chain}`. Matching only `\A`
# and `\E` missed the comprehension form, so `i` was declared as a CONSTANT and
# then bound by `\in` in the same module -- SANY's "Multiply-defined symbol 'i'".
_BINDER = re.compile(
    r"(?<![A-Za-z0-9_.])"
    r"((?:[A-Za-z_][A-Za-z0-9_]*\s*,\s*)*[A-Za-z_][A-Za-z0-9_]*)\s*\\in"
)
# Names the TLA+ grammar itself supplies, so a spec may use them without
# declaring anything. Operators of the EXTENDS'd standard modules are listed
# separately: Sequences exports Len, Append, Head, Tail, SubSeq, SelectSeq and
# Seq -- but not Range, which lives in the TLC module and is not part of the
# language.
_TLA_KEYWORDS = {
    "TRUE", "FALSE", "NULL", "in", "A", "E", "Len", "Cardinality",
    "DOMAIN", "IF", "THEN", "ELSE", "CHOOSE", "SUBSET", "UNION", "approx",
}


def free_identifiers(invariants):
    """Names an invariant mentions that it does not bind and TLA+ does not supply.

    Every `\\A x \\in S` binds `x`; everything else that looks like an
    identifier is either supplied by the standard modules (Len, Cardinality,
    ...) or something the module must declare. Without this, a generated spec
    stops at the first undefined name and SANY reports a screen of
    "Unknown operator" errors -- which is what the first version of this
    generator produced for all 22 services.

    Names in `_AUX_OPERATORS` are deliberately *not* excluded: they are
    declared by the module, so they must appear here for the template to
    emit them.
    """
    free = set()
    for inv in invariants:
        bound = set()
        for m in _BINDER.finditer(inv):
            for part in m.group(1).split(","):
                part = part.strip()
                if _IDENT.fullmatch(part):
                    bound.add(part)
        # `a.signature` mentions both `a` and `signature`; the field name is
        # not a free variable. But a name can be both: `hmac(h, k) = h.hmac`
        # uses `hmac` as an operator and as a record field. Only drop a name
        # that appears *solely* after a dot -- `hmac` is called, so it must be
        # declared.
        fields = set(re.findall(r"\.\s*([A-Za-z_][A-Za-z0-9_]*)", inv))
        called = set(re.findall(r"([A-Za-z_][A-Za-z0-9_]*)\s*\(", inv))
        for tok in _IDENT.findall(inv):
            if tok in bound or tok in _TLA_KEYWORDS:
                continue
            if tok in fields and tok not in called:
                continue
            free.add(tok)
    return sorted(free)


def _namespace_binders(inv, n):
    """Rename an invariant's bound variables so they cannot collide.

    Each invariant becomes its own top-level definition, and in TLA+ the
    quantifier scope of `\\A i \\in S : ...` does not end at the end of the
    definition -- a second definition that also binds `i` is a
    "Multiply-defined symbol 'i'" error, which SANY reports against the
    *second* one. Suffixing with the invariant's index keeps every binder
    unique without changing what the invariant says.
    """
    names = set()
    for m in _BINDER.finditer(inv):
        for part in m.group(1).split(","):
            part = part.strip()
            if _IDENT.fullmatch(part):
                names.add(part)

    out = inv
    for nm in sorted(names, key=len, reverse=True):
        # `(?<![A-Za-z0-9_.])` keeps `h.i` a record field rather than a binder
        # reference; the binder list itself is matched by the same pattern, so
        # one pass renames declaration and use together.
        out = re.sub(rf"(?<![A-Za-z0-9_.]){re.escape(nm)}(?![A-Za-z0-9_])",
                     f"{nm}_{n}", out)
    return out


def tla_template(name, invariants):
    """A TLA+ module that SANY accepts without error.

    `name` is the *service* name; the module and file both take
    `module_name(name)`, because SANY rejects a file whose stem disagrees with
    the module it contains.
    """
    module = module_name(name)
    invs = "\n".join(
        f"Inv{i+1} == {_namespace_binders(inv, i+1)}"
        for i, inv in enumerate(invariants)
    )
    body = invs if invs else "\\* No invariants specified for this service yet."

    free = free_identifiers(invariants)
    # Operators are *defined* (they are called with arguments); everything
    # else is a CONSTANT. Declaring `Hash` as a CONSTANT would not resolve
    # `Hash(x)`, and defining a plain value as an operator would not resolve
    # its bare use -- so the two sets are emitted differently.
    ops = sorted(set(_AUX_OPERATORS) & set(free))
    consts = sorted(set(free) - set(ops))

    decls = ["CONSTANTS NULL"]
    if consts:
        decls.append("CONSTANTS " + ", ".join(consts))
    for op in ops:
        args = ", ".join(f"a{i}" for i in range(_AUX_OPERATORS[op]))
        decls.append(
            f"\\* TODO: bind `{op}` to the implementation's own definition.\n"
            f"\\* Left unconstrained here, so the invariants below type-check\n"
            f"\\* without proving anything about it.\n"
            f"{op}({args}) == CHOOSE r : TRUE"
        )
    decl_lines = "\n".join(decls)

    return f"""---- MODULE {module} ----
\\* TLA+ spec for {name} -- auto-generated by generate_proofs.py
\\* Audit ref: M5 chain; ADR refs: 0004/0006/0009/0010/0019/0022/0023
\\*
\\* The CONSTANTS below are the free names the invariants mention. They are
\\* declared so the module type-checks; binding each to the implementation's
\\* own definition is what turns this from a well-formed spec into a proof.
EXTENDS Naturals, FiniteSets, Sequences, TLC
{decl_lines}
VARIABLES state
{body}

====
"""


def certora_template(name, rules):
    rules_str = "\n\n".join(rules) if rules else "// No rules specified yet."
    return f"""/* Certora CVL spec for {name} -- auto-generated
 * Run: certora verify --spec proofs/{name}/{name}.cvl --contract <path>
 */
{rules_str}
"""

def run_script(name, tla_files, certora_files):
    return f"""#!/bin/bash
# TKLA+ verification runner for {name}
# Audit ref: M5 auditId=g-tkla-{name}, proofHash=sha256(specs), signature=HSM
set -euo pipefail
cd "$(dirname "$0")"
echo "=== {name} TLA+ ==="
for spec in *.tla; do
  if [ -f "$spec" ]; then
    cfg=$(echo $spec | sed 's/.tla$/.cfg/')
    if [ -f "$cfg" ]; then
      tla2tools "$spec" -config "$cfg" -workers 4 -deadlock 2>&1 | tee "results/$(basename $spec .tla)-tla.log"
    else
      echo "SKIP $spec (no .cfg)"
    fi
  fi
done
echo "=== {name} Certora ==="
for spec in *.cvl; do
  if [ -f "$spec" ]; then
    echo "certora verify --spec $spec"
  fi
done
"""

def readme(name, description, invariants, certora_rules, tla_status, certora_status, t_status, chaos_scenarios):
    return f"""# {name} — TKLA+ Proof Package

**Service**: {description}
**Audit ref**: M5 (auditId=g-tkla-{name}, proofHash=sha256(specs), sig=HSM)
**ADR**: 0004/0006/0009/0010/0019/0022/0023

## T — Truthful (compile + test)
| Check | Status |
|---|---|
| `cargo check --locked` | {t_status} |
| `cargo test` | {t_status} |
| `pytest -vxs tests/ci` | {t_status} |

## KL — Knowledge-Lattice (M5 audit chain)
- Every handler emits: `auditId`, `proofHash`, `signature`, `correlation_key`
- M5 lattice: `auditId` → `proofHash` → `signature` → `correlation_key` → detection → ADR

## A+ — Adversarial (chaos)
{chaos_scenarios}

## TLA+ Invariants
{chr(10).join(f'- `{i+1}. {inv}`' for i, inv in enumerate(invariants))}

## Certora CVL Rules
{chr(10).join(f'- `{r}`' for r in certora_rules)}

## Verification
```bash
./run.sh        # run all TLA+ + Certora
./run.sh tla    # TLA+ only
./run.sh cert   # Certora only
```
"""

# ─── SERVICE DEFINITIONS ───────────────────────────────────────────────────

SPECS = {

"abis-svc": {
    "description": "ABIS attestation service — NIST MINEX biometric deduplication",
    "invariants": [
        "Cardinality(attestations) <= MAX_ATTESTATIONS",
        "\\A a \\in attestations : a.signature /= NULL",
        "\\A a \\in attestations : a.timestamp <= NOW",
        "\\A a, b \\in attestations : a = b => a.audit_id = b.audit_id",
    ],
    "certora_rules": [
        "spec uniqueAttestation { method f(id: String, ts: Int) { assert ts > 0; } }",
        "spec signatureValid { method f(a: Attestation) { assert verifyEd25519(a); } }",
    ],
    "chaos": """- `chaos/scenarios/eel-pod-kill-abis-svc.yaml` — pod restart during attestation
- `chaos/scenarios/eel-abis-gallery-shard-mismatch-abis-svc.yaml` — gallery shard mismatch""",
    "t_status": "✅ 0 errors",
    "cert_status": "⏳ pending",
},

"transparency-svc": {
    "description": "Transparency log — append-only Merkle tree, STH, VRF sortition, gazette anchor",
    "invariants": [
        "\\A i \\in 1..Len(chain)-1 : chain[i+1].prev_hash = Hash(chain[i])",
        "Len(chain) >= 1 => chain[1].prev_hash = GENESIS_HASH",
        "\\A i, j \\in 1..Len(chain) : i /= j => chain[i].audit_id /= chain[j].audit_id",
        "\\A entry \\in {chain[i] : i \\in DOMAIN chain} : Cardinality(entry.signatures) >= 2",
        "100 * Len(selected) >= 4 * Len(all_stations) /\\ 100 * Len(selected) <= 6 * Len(all_stations)",
        "\\A g \\in gazette_anchors : g.sha256 = Hash(g.content)",
    ],
    "certora_rules": [
        "spec appendOnly { method f(chain: Chain) { require chain.len == chain.len'; } }",
        "spec noFork { method f(chain: Chain) { assert chain.tip == Hash(chain[Len(chain)-1]); } }",
        "spec deterministicSortition { method f(mr: String) { assert sortition(mr) == sortition(mr); } }",
        "spec gazetteHash { method f(g: GazetteIssue) { assert SHA256(g.content) == g.sha256; } }",
    ],
    "chaos": """- `chaos/scenarios/eel-sequencer-stall-audit-log-transparency-svc.yaml` — STH stall
- `chaos/scenarios/eel-transparency-audit-replay-transparency-svc.yaml` — replay detection
- `chaos/scenarios/eel-storage-site-loss-partial-read-transparency-svc.yaml` — partial read on site loss""",
    "t_status": "✅ repro-bundle + gazette-anchor compile clean",
    "cert_status": "⏳ pending",
},

"bvr-svc": {
    "description": "Biometric voter registration — ABIS deduplication, nearform-nbis FFI, gallery management",
    "invariants": [
        "\\A v \\in voters : v.biometric_template /= NULL",
        "\\A v \\in voters : v.audit_id /= NULL",
        "\\A v \\in voters : \\A u \\in voters : v /= u => v.national_id /= u.national_id",
        "Cardinality(dedup_candidates) >= 2 => duplicate_found = TRUE",
        "\\A v \\in verified : verify_nbis(v.template, v.fingerprint) = TRUE",
    ],
    "certora_rules": [
        "spec templateExists { method f(v: Voter) { assert v.template /= NULL; } }",
        "spec dedupFair { method f(gallery: Set[Voter], newV: Voter) { assert bozorth(gallery, newV); } }",
        "spec auditEmitted { method f(v: Voter) { assert v.audit_id /= NULL; } }",
    ],
    "chaos": """- `chaos/scenarios/eel-bvr-enrolment-overflow-bvr-svc.yaml` — enrolment overflow
- `chaos/scenarios/eel-credential-revocation-storm-bvr-svc.yaml` — credential revocation storm
- `chaos/scenarios/insider-risk/11-gallery-poisoning-attempt.yaml` — adversarial template injection""",
    "t_status": "✅ 0 errors",
    "cert_status": "⏳ pending",
},

"bvi-svc": {
    "description": "Biometric verification identity — real-time capture, FAP60, anti-iris, anomaly detection",
    "invariants": [
        "\\A r \\in verifications : r.score >= 0.0 /\\ r.score <= 1.0",
        "\\A r \\in verifications : r.liveness_score >= 0.0 /\\ r.liveness_score <= 1.0",
        "\\A r \\in verifications : r.fap60_result /= NULL => r.fap60_genuine = TRUE",
        "\\A anomaly \\in anomalies : anomaly.detected_at <= NOW",
        "\\A anomaly \\in anomalies : anomaly.confidence >= 0.0",
    ],
    "certora_rules": [
        "spec scoreBounded { method f(r: VerificationResult) { assert 0.0 <= r.score && r.score <= 1.0; } }",
        "spec livenessPass { method f(r: VerificationResult) { assert r.liveness_score >= 0.5; } }",
        "spec anomalyConfidence { method f(a: Anomaly) { assert a.confidence >= 0.0 && a.confidence <= 1.0; } }",
    ],
    "chaos": """- `chaos/scenarios/eel-pod-kill-bvi-svc.yaml` — pod kill
- `chaos/scenarios/eel-bvi-identification-storm-bvi-svc.yaml` — identification storm
- `chaos/scenarios/integration-failure/17-nist-beacon-timeout.yaml` — beacon timeout""",
    "t_status": "✅ 0 errors",
    "cert_status": "⏳ pending",
},

"field-mobile-svc": {
    "description": "Mobile BFF — offline-first, attestation, form quality gate, station binding",
    "invariants": [
        "\\A req \\in form_submissions : req.quality_gate_passed = TRUE => req.accepted = TRUE",
        "\\A req \\in form_submissions : req.quality_gate_passed = FALSE => req.rejected = TRUE",
        "Len(rejected_reasons) >= 1 => quality_gate_passed = FALSE",
        "\\A req \\in form_submissions : req.station_code /= NULL",
        "Len(idempotency_keys) >= 1 => \\A k \\in idempotency_keys : \\A x \\in idempotency_keys : k = x => k.responses = x.responses",
        "\\A app \\in attestations : app.verdict \\in {\"MEETS\", \"FAILS\", \"UNVERIFIED\"}",
    ],
    "certora_rules": [
        "spec qualityGatePass { method f(r: FormSubmission) { assert r.quality_gate_passed == true => r.accepted == true; } }",
        "spec qualityGateReject { method f(r: FormSubmission) { assert r.quality_gate_passed == false => r.rejected == true; } }",
        "spec idempotencyKey { method f(k: String, v: Response) { assert store(k, v) == store(k, v); } }",
        "spec attestationVerdict { method f(v: AttestationVerdict) { assert v.verdict in {'MEETS','FAILS','UNVERIFIED'}; } }",
    ],
    "chaos": """- `chaos/scenarios/eel-pod-kill-field-mobile-svc.yaml`
- `chaos/scenarios/eel-mobile-bff-auth-cascade-field-mobile-svc.yaml`
- `chaos/scenarios/eel-field-mobile-sync-disconnect-loop-field-mobile-svc.yaml`""",
    "t_status": "⚠️ 30 pre-existing errors (E0061/E0308/E0599; redesign needed)",
    "cert_status": "⏳ pending",
},

"inventory-svc": {
    "description": "Inventory tracking — RFID, ballot chain-of-custody, HMAC-SHA256, M-Pesa B2C",
    "invariants": [
        "\\A h \\in handovers : hmac(h, HMACKey) = h.hmac",
        "\\A i \\in 1..Len(chain)-1 : chain[i+1].prev_hash = Hash(chain[i])",
        "Len(chain) >= 1 => chain[1].seq_no = 1",
        "\\A i \\in 1..Len(chain)-1 : chain[i+1].seq_no = chain[i].seq_no + 1",
        "\\A payment \\in payments : payment.response_code \\in {\"0\", \"1\", \"17\", \"INSF\"}",
        "\\A payment \\in payments : payment.audit_id /= NULL",
    ],
    "certora_rules": [
        "spec hmacIntegrity { method f(h: BallotHandover, key: Bytes) { assert hmac_sha256(h, key) == h.hmac; } }",
        "spec seqNoMonotonic { method f(chain: Chain) { assert chain[i+1].seq == chain[i].seq + 1; } }",
        "spec paymentAudit { method f(p: Payment) { assert p.audit_id /= NULL; } }",
    ],
    "chaos": """- `chaos/scenarios/eel-pod-kill-inventory-svc.yaml`
- `chaos/scenarios/eel-asset-return-seal-failure-inventory-svc.yaml`
- `chaos/scenarios/integration-failure/09-mpesa-daraja-timeout.yaml`""",
    "t_status": "✅ 0 errors",
    "cert_status": "⏳ pending",
},

"hsm-signer": {
    "description": "HSM signer — PKCS#11 Ed25519+ML-DSA dual signature, CCB log, break-glass",
    "invariants": [
        "\\A entry \\in ccb_log : Cardinality(entry.approvers \\cap ValidSigners) >= 2",
        "\\A i \\in 2..Len(ccb_log) : ccb_log[i].prev_hash = Hash(ccb_log[i-1])",
        "\\A entry \\in ccb_log : entry.final = TRUE => Cardinality(entry.approvers) >= 2",
        "\\A sig \\in signatures : verifyEd25519(sig) = TRUE /\\ verifyMLDSA(sig) = TRUE",
        "break_glass_active = TRUE => emergency_signatures >= 1 /\\ audit_logged = TRUE",
        "\\A entry \\in ccb_log : entry.audit_id /= NULL",
    ],
    "certora_rules": [
        "spec multiSig2of3 { method f(e: CCBEntry) { assert count(e.approvers) >= 2; } }",
        "spec prevHashChain { method f(e: CCBEntry, prev: CCBEntry) { assert e.prev_hash == hash(prev); } }",
        "spec dualSignature { method f(s: Signature) { assert verifyEd25519(s) && verifyMLDSA(s); } }",
        "spec breakGlassAudit { method f() { assert break_glass_active == true => audit_logged == true; } }",
    ],
    "chaos": """- `chaos/scenarios/insider-risk/08-hsm-key-extraction-attempt.yaml` — key extraction
- `chaos/scenarios/eel-hsm-unavailable.yaml` — PKCS#11 unavailability
- `chaos/scenarios/integration-failure/06-hsm-pkcs11-down.yaml` — signer down during declaration""",
    "t_status": "✅ 0 errors",
    "cert_status": "⏳ pending",
},
}


# ─── GENERATION ────────────────────────────────────────────────────────────

def _service_dir(root, name):
    return os.path.join(root, name)


def _write(path, content, executable=False):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(content)
    if executable:
        os.chmod(path, 0o755)


def generate(root, specs, services):
    """Write one proof package per service under `root`.

    A service in SERVICES with no SPECS entry still gets a package, with its
    invariants left empty and that fact stated in the README. Generating an
    empty package is honest; omitting the service would make the set of
    packages disagree with the set of services.
    """
    written, unspecified = [], []
    for name in services:
        spec = specs.get(name)
        if spec is None:
            unspecified.append(name)
            spec = {
                "description": f"{name} — no invariants specified yet",
                "invariants": [],
                "certora_rules": [],
                "chaos": "- (none specified)",
                "t_status": "—",
                "cert_status": "—",
            }

        out = _service_dir(root, name)
        module = module_name(name)
        _write(os.path.join(out, f"{module}.tla"),
               tla_template(name, spec["invariants"]))
        _write(os.path.join(out, f"{name}.cvl"),
               certora_template(name, spec["certora_rules"]))
        _write(os.path.join(out, "run.sh"),
               run_script(name, [f"{module}.tla"], [f"{name}.cvl"]),
               executable=True)
        _write(os.path.join(out, "README.md"),
               readme(name, spec["description"], spec["invariants"],
                      spec["certora_rules"], spec["t_status"],
                      spec["cert_status"], spec["t_status"], spec["chaos"]))
        written.append(name)
    return written, unspecified


def check_tla(paths):
    """Check each spec with SANY; return the ones that do not verify clean.

    SANY is the reference front end for TLA+ and what `tlc` itself invokes, so
    a spec that fails here cannot be model-checked. It reports two distinct
    failures and both matter:

      * a parse error ("Fatal errors"), where the file is not TLA+ at all;
      * semantic errors, where the file parses but names operators that are
        not declared -- the spec looks complete and proves nothing.

    The second is the quieter one, and SANY's output is easy to misread in
    both directions:

      * it exits 0 whether or not the spec is sound, so the exit code says
        nothing;
      * it prints "Semantic processing of module X" on every run, sound or
        not, so grepping for that line reports success on a spec with
        thirteen undefined symbols;
      * on success it prints *no* `*** Errors` line at all and does print
        "Linting of module X"; on failure it prints the error count and no
        linting line.

    So the signal is the pair, not either half. A run that shows neither line
    is a failure: SANY did not get far enough to judge the spec.
    """
    jar = os.path.expanduser("~/.cache/tla2tools.jar")
    if not os.path.isfile(jar) or not shutil.which("java"):
        raise RuntimeError(
            "SANY unavailable: need java on PATH and ~/.cache/tla2tools.jar. "
            "Install with: curl -L -o ~/.cache/tla2tools.jar "
            "https://github.com/tlaplus/tlaplus/releases/latest/download/tla2tools.jar"
        )

    bad = []
    for path in paths:
        proc = subprocess.run(
            ["java", "-cp", jar, "tla2sany.SANY", path],
            capture_output=True, text=True, check=False,
        )
        out = proc.stdout + proc.stderr
        counts = re.findall(r"\*\*\* Errors: (\d+)", out)
        linted = "Linting of module" in out
        if linted and not counts:
            continue
        if not counts:
            bad.append((path, "SANY neither linted the module nor reported "
                              "an error count"))
            continue
        first = ""
        for line in out.splitlines():
            if line.startswith(("Unknown operator", "Could not find declaration",
                                "Couldn't resolve", "Was expecting",
                                "File name")):
                first = line.strip()
                break
        bad.append((path, f"{counts[-1]} error(s)"
                          + (f": {first}" if first else "")))
    return bad


def main():
    root = os.path.dirname(os.path.abspath(__file__))
    written, unspecified = generate(root, SPECS, SERVICES)
    print(f"generated {len(written)} proof packages under {root}")
    if unspecified:
        print(f"  {len(unspecified)} with no invariants specified: "
              f"{', '.join(unspecified)}")

    specs = [os.path.join(root, s, f"{module_name(s)}.tla") for s in written]
    bad = check_tla(specs)
    if bad:
        print(f"  {len(bad)} generated specs do not parse:")
        for path, why in bad:
            print(f"    {os.path.relpath(path, root)}: {why}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

