# operations/e-learning

Learning Management System (LMS) content and approval workflow for eel. Polling officers,
party agents, and observers complete mandatory e-learning modules before receiving their
access credentials. Completion records are verified by `e-learning-svc` before credential
issuance.

## Purpose

Kenya election law requires trained and accredited personnel at every polling station.
eel enforces this programmatically: a polling officer's kit credential is not issued
until their `e-learning-svc` completion record shows all mandatory modules passed at
the required score threshold.

## Module catalog

| Module | Audience | Required score | Estimated time |
|--------|----------|----------------|----------------|
| BVI identification procedure | Identification officers | 80% | 45 min |
| RTS results transmission | Results officers | 85% | 60 min |
| BVR voter registration | Registration officers | 80% | 60 min |
| Polling station security | All field staff | 75% | 30 min |
| Party agent rights and responsibilities | Party agents | 75% | 30 min |
| Observer code of conduct | Observers | 75% | 20 min |
| Kit handling and troubleshooting | All kit users | 80% | 45 min |

## Contents

| File | Description |
|------|-------------|
| `content-approval-workflow.md` | Approval process for new and updated module content |
| `README.md` | This file |

## Usage

```bash
# Check completion record for an officer (via e-learning-svc API)
curl -H "Authorization: Bearer $ADMIN_TOKEN" \
  https://e-learning.eel.internal/api/v1/completions?officer_id=<uuid>

# Trigger credential issuance check (called by credential-svc after completion)
curl -X POST \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  https://e-learning.eel.internal/api/v1/verify \
  -d '{"officer_id":"<uuid>","required_modules":["bvi-procedure","rts-transmission"]}'

# Export completion report for a county
curl -H "Authorization: Bearer $ADMIN_TOKEN" \
  "https://e-learning.eel.internal/api/v1/reports/county/<county-code>/completions" \
  -o reports/county-<county-code>-completions.csv
```

## Config and environment

| Variable | Description |
|----------|-------------|
| `ELEARNING_URL` | Base URL for `e-learning-svc` API |
| `ADMIN_TOKEN` | Admin bearer token; retrieve from vault |
| `SCORE_THRESHOLD_*` | Per-module pass score thresholds (set in `e-learning-svc` config) |

## Dependencies

- `e-learning-svc`: serves LMS content and tracks completion records
- `credential-svc` (or equivalent): calls `e-learning-svc` to verify completion before
  issuing kit credentials
- LMS content authoring tool: modules are authored externally and imported via the content
  approval workflow (`content-approval-workflow.md`)

## Test and verify

```bash
# Verify e-learning-svc is healthy
curl https://e-learning.eel.internal/healthz

# Create a test completion record and verify retrieval
curl -X POST \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  https://e-learning.eel.internal/api/v1/completions \
  -d '{"officer_id":"test-uuid","module":"bvi-procedure","score":90,"passed":true}'

curl -H "Authorization: Bearer $ADMIN_TOKEN" \
  "https://e-learning.eel.internal/api/v1/completions?officer_id=test-uuid" \
  | jq '.[0].passed'
# Expected: true
```

## Related docs

- `operations/README.md`: operations overview
- `operations/mdm/README.md`: MDM device enrolment happens after e-learning completion
- `operations/helpdesk/README.md`: e-learning access issues routed to helpdesk

## Limitations

- Module content is managed outside this repo; only the approval workflow is documented here.
- Completion records are the authoritative source for credential issuance; incorrect records
  require manual correction via the `e-learning-svc` admin API with appropriate audit log entry.
- The LMS does not support offline completion; officers must have internet access to complete
  modules before election day.
