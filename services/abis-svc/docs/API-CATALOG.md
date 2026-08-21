# abis-svc API Catalog

Base URL: `https://abis.eel.internal` (internal) or `http://localhost:8080` (dev).

Auth: SPIFFE SVID mTLS (all endpoints). Rate limits enforced by Kong gateway.

Response signing: all 2xx responses carry `X-Eel-Signature: ed25519:<hex>` over `{status}:{body_sha256}`.

---

## Health

### GET /healthz

Auth: none. Rate limit: unlimited.

Returns service health. Used by Kubernetes liveness probe.

```
curl http://localhost:8080/healthz
{"ok":true}
```

---

## Match

### POST /api/v1/abis/match/1n

1:N deduplication match. Async; returns job ID.

Auth: SVID (enrolment-svc, kit, abis-api). Idempotency: required (`Idempotency-Key` header). Rate limit: 100 rps per caller.

Request:
```json
{
  "probe_ref": "bio-store://eel.local/probes/<uuid>",
  "template_blob": "<base64>",
  "modality": "fingerprint",
  "caller_id": "spiffe://eel.local/ns/enrolment/sa/enrolment-svc",
  "election_id": "<uuid7>",
  "subject_ref": "subject:<id>"
}
```

Response 202:
```json
{"job_id": "<uuid7>", "status": "queued", "estimated_complete_at": "<iso8601>"}
```

```
curl -X POST http://localhost:8080/api/v1/abis/match/1n \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: $(uuidgen)" \
  -d '{"probe_ref":"bio-store://eel.local/probes/test","template_blob":"AAAA","modality":"fingerprint","caller_id":"spiffe://eel.local/ns/enrolment/sa/enrolment-svc","election_id":"00000000-0000-7000-8000-000000000001"}'
```

---

### POST /api/v1/abis/match/11

1:1 synchronous match. Returns score and decision inline.

Auth: SVID (bvi-backend). Idempotency: required. Rate limit: 500 rps.

Request:
```json
{
  "probe_ref": "bio-store://eel.local/probes/<uuid>",
  "gallery_ref": "bio-store://eel.local/gallery/voter-<id>",
  "template_blob": "<base64>",
  "modality": "fingerprint",
  "caller_id": "spiffe://eel.local/ns/bvi/sa/bvi-backend"
}
```

Response 200:
```json
{"match": true, "score": 0.923, "decision": "accept", "latency_ms": 18}
```

---

### GET /api/v1/abis/match/{job_id}

Poll 1:N job result.

Auth: SVID. Rate limit: 200 rps.

Response 200 (complete):
```json
{"job_id":"<uuid7>","status":"complete","decision":"clean","score":null,"candidates":[]}
```

Response 202 (pending): `{"job_id":"<uuid7>","status":"pending"}`.

---

## Batch

### POST /api/v1/abis/batch/dedup

Submit batch deduplication. Accepts NDJSON of enrolment records.

Auth: SVID (batch-processor). Idempotency: required. Rate limit: 10 rps. Max body: 10 MB.

---

## Adjudication

### GET /api/v1/abis/adjudication

List open adjudication cases. Paginated (`?page=1&per_page=50`).

Auth: SVID (adjudicator, console). Rate limit: 50 rps.

### PATCH /api/v1/abis/adjudication/{case_id}

Update case status or assignment.

Auth: SVID (adjudicator). Idempotency: required. Rate limit: 20 rps.

---

## Reports

### GET /api/v1/abis/reports

Query operational reports. `?type=fmr|fnmr|fairness|recall|snapshot-integrity`.

Auth: SVID (reporting-svc, console). Rate limit: 10 rps.
