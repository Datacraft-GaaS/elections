# Completeness attestation flow

After all 9 completeness audit steps pass, `steps/9.9-report.py` generates
`completeness-attestation.json`. This document describes the tlog submission flow.

## Steps

1. `run-audit.sh` invokes steps 9.1 through 9.8 and aggregates results into
   `completeness-report.json`.

2. `steps/9.9-report.py` renders `completeness-report.md` and computes:
   ```
   sha256 = SHA-256(JSON.stringify(completeness-report.json, sort_keys=true))
   ```

3. The script submits a `meta-event` leaf to the transparency log:
   ```
   POST ${EEL_TLOG_URL}/api/v1/log/append
   {
     "kind": "meta-event",
     "subject": "completeness-audit",
     "sha256": "<hex>",
     "timestamp": "<ISO8601>"
   }
   ```
   `EEL_TLOG_URL` defaults to `http://localhost:3000` (transparency-svc).

4. The response includes an inclusion proof (Merkle path + root).
   This is stored in `completeness-attestation.json`:
   ```json
   {
     "version": "1",
     "generated_at": "<ISO8601>",
     "report_sha256": "<hex>",
     "inclusion_proof": { ... }
   }
   ```

5. Gate G6 reviewers verify the proof via:
   ```
   GET /api/v1/log/inclusion/<leaf-hash>
   ```
   and confirm the leaf matches `completeness-report.json`.

6. The attestation JSON is committed to the audit record and linked from
   `docs/audit/gate-G6.md`.

## Offline / air-gapped

If the transparency log is not reachable (air-gapped signing ceremony),
run `9.9-report.py` with `EEL_TLOG_URL=` empty. The script writes
`inclusion_proof: null` and prints instructions for manual submission.
The manual submission must be recorded before Gate G6 closes.
