# KNADS Long-Term Archive API

Kenya National Archives and Documentation Service REST API for long-term preservation of election records, documents, and datasets. Handles document ingestion, retrieval, and preservation verification.

## MoU Status

KNADS MoU signed. API access and credentials issued for production archival operations. Annual compliance audit required.

## Endpoint

Production: https://api.knads.go.ke/v1/
Sandbox: https://api-dev.knads.go.ke/v1/

## Protocol and Authentication

REST over HTTPS with JSON payloads. Authentication via X-API-Key header.

**API Key:**
Issued by KNADS ICT department. Required on all requests.

## Request and Response Shape

**Record Ingest:**
Request: `POST /ingest {collection_id, document_title, document_type, content_type, metadata: {creator, subject, date_range}, payload: {data: "base64-encoded-bytes or JSON"}}`
Response: `{ingest_id, status: "ACCEPTED|PROCESSING|ARCHIVED", archival_reference, preservation_url, ingest_timestamp}`

**Record Retrieve:**
Request: `GET /records/{archival_reference}`
Response: `{archival_reference, document_title, document_type, content_type, metadata, preservation_format, retrieval_url, access_restrictions}`

**Preservation Check:**
Request: `GET /records/{archival_reference}/preservation-status`
Response: `{status: "VERIFIED|DEGRADED|LOST", last_verified_date, verification_method, fixity_hash_sha256}`

## Sample Curl

```bash
curl -H "X-API-Key: $KNADS_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"collection_id":"election-2027","document_title":"Polling Results Bundle","document_type":"dataset","content_type":"application/zip","metadata":{"creator":"eel-system","subject":"election-results","date_range":"2024-08-21"}}' \
  https://api.knads.go.ke/v1/ingest

curl -H "X-API-Key: $KNADS_API_KEY" \
  https://api.knads.go.ke/v1/records/KNADS-2024-00001
```

## Health Probe

GET /health returns 200 OK if service is operational.

## Rate Limits

30 requests/minute per API key. Bulk ingestion (multiple records) accepted but each record counts toward rate limit.

## PII Enforcement

Election records may contain voter data (restricted tier-A). Archival metadata is tier-B internal. KNADS maintains strict access controls on ingested data. Logs redact document titles and metadata if they contain PII. Retrieval restricted to authorized operators only.

## Chaos Toggle

KNADS_CHAOS=true enables chaos injection. Causes random ingestion rejections and preservation verification failures (test only).

## Fallback Behavior

On adapter unavailability, record ingest queued locally as BagIt package. Manual archival job (monthly) handles bulk transmission to KNADS. Local backup archive (cold storage) managed via M69 infrastructure.

## Circuit Breaker Policy

Threshold: 3 consecutive failures
Open duration: 60 seconds
Rationale: KNADS is government archival system with periodic maintenance windows
Half-open: permits 1 probe request

## Environment Variables

KNADS_BASE_URL: KNADS API base URL
KNADS_API_KEY: API key for X-API-Key header
KNADS_COLLECTION_ID: Default collection ID for ingestions
KNADS_TIMEOUT_SEC: Request timeout in seconds (default: 60)
KNADS_CHAOS: "true" enables chaos (test only)

## Related Modules

M69 (long-term-archive ingest and retrieval), M14 (reproducibility-bundle DOI and archival)

## Runbook

docs/runbooks/knads-archival-management.md (record ingest workflow, retrieval, preservation verification, BagIt package management)

