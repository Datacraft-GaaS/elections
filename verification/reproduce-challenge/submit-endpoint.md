# Reproduce-Challenge HTTP API Contract

Base URL: `https://api.eel-verification.io`

All timestamps are ISO 8601 UTC. All endpoints use TLS 1.3 minimum.

---

## Authentication

Registered teams receive a Bearer token via the programme coordinator. Include it in every mutating request.

```
Authorization: Bearer <team_token>
```

GET endpoints for public results require no authentication.

---

## Endpoints

### Submit a Reproduction

```
POST /v1/reproduce-challenge/submissions
```

**Request**

Content-Type: `multipart/form-data`

| Field | Type | Required | Description |
|---|---|---|---|
| `team_id` | string | yes | Your registered team identifier, e.g. `eel-test-team` |
| `day` | string | yes | Verification day in `YYYY-MM-DD` format |
| `submission.tar.gz` | binary | yes | Tarball containing `manifest.json`, `tally.json`, `events.ndjson` |

**Response 202 Accepted**

```json
{
  "submission_id": "01950e7a-b2f1-7000-8000-abcdef012345",
  "team_id": "eel-test-team",
  "day": "2024-08-01",
  "received_at": "2024-08-01T03:14:00Z",
  "status": "queued",
  "status_url": "/v1/reproduce-challenge/submissions/01950e7a-b2f1-7000-8000-abcdef012345/status"
}
```

The submission enters an async verification queue. Poll the status URL for results.

---

### Poll Submission Status

```
GET /v1/reproduce-challenge/submissions/{submission_id}/status
```

**Path parameter:** `submission_id` as returned by the submit endpoint.

**Auth:** Bearer token (team can only read its own submissions).

**Response 200 OK**

```json
{
  "submission_id": "01950e7a-b2f1-7000-8000-abcdef012345",
  "team_id": "eel-test-team",
  "day": "2024-08-01",
  "status": "verified",
  "result": "PASS",
  "verified_at": "2024-08-01T04:02:17Z",
  "report_url": "/v1/reproduce-challenge/results/2024-08-01/eel-test-team/report.md"
}
```

`status` values: `queued`, `processing`, `verified`, `rejected`.

`result` values: `PASS`, `FAIL`, `ERROR` (only present when `status` is `verified` or `rejected`).

---

### Fetch Day Results (Public)

```
GET /v1/reproduce-challenge/results/{day}
```

**Path parameter:** `day` in `YYYY-MM-DD` format.

**Auth:** None. Public endpoint.

**Response 200 OK**

```json
{
  "day": "2024-08-01",
  "published_at": "2024-08-01T07:58:00Z",
  "reference_sha256": "e3b0c44298fc1c149afbf4c8996fb924...",
  "teams": [
    {
      "team_id": "eel-test-team",
      "result": "PASS",
      "submission_sha256": "2cf24dba5fb0a30e26e83b2ac5b9e29e...",
      "verified_at": "2024-08-01T04:02:17Z",
      "report_url": "/v1/reproduce-challenge/results/2024-08-01/eel-test-team/report.md"
    }
  ]
}
```

Returns 404 if results for `day` are not yet published.

---

## Error Codes

| HTTP Status | Code | Meaning |
|---|---|---|
| 400 | `MISSING_FIELD` | Required multipart field absent |
| 400 | `INVALID_DAY` | `day` not parseable as `YYYY-MM-DD` |
| 400 | `WINDOW_CLOSED` | Submission received after the 8-hour window |
| 401 | `UNAUTHORIZED` | Missing or invalid Bearer token |
| 403 | `TEAM_MISMATCH` | Token's team does not match `team_id` field |
| 409 | `DUPLICATE_SUBMISSION` | Team already submitted for this day |
| 413 | `PAYLOAD_TOO_LARGE` | Tarball exceeds 200 MB |
| 422 | `MANIFEST_MISSING` | `manifest.json` absent from tarball |
| 422 | `MANIFEST_INVALID` | `manifest.json` fails schema validation |
| 429 | `RATE_LIMITED` | See rate limits below |
| 500 | `INTERNAL_ERROR` | Contact programme coordinator |
| 503 | `WINDOW_NOT_OPEN` | Submission window has not opened yet |

All error responses share this envelope:

```json
{
  "error": {
    "code": "WINDOW_CLOSED",
    "message": "The submission window for 2024-08-01 closed at 10:00 EAT.",
    "request_id": "req_01950e7b..."
  }
}
```

---

## Rate Limits

| Scope | Limit |
|---|---|
| Submit (`POST /submissions`) | 3 requests per day per team |
| Status poll | 60 requests per minute per token |
| Public results | 120 requests per minute per IP |

Rate limit headers are returned on every response:

```
X-RateLimit-Limit: 3
X-RateLimit-Remaining: 2
X-RateLimit-Reset: 1722470400
```

A `429` response includes a `Retry-After` header in seconds.

---

## Idempotency

The submit endpoint is NOT idempotent by default. A second submission for the same `(team_id, day)` pair returns `409 DUPLICATE_SUBMISSION`. To replace a prior submission within the window, append `?replace=true`; the earlier submission is then archived and the new one enters the queue.
