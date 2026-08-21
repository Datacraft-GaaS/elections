# DataCite DOI Registration API

DataCite international digital object identifier (DOI) registration for election datasets and reproducibility bundles. Handles DOI minting, metadata updates, and public DOI resolution.

## MoU Status

DataCite member agreement with annual subscription fee. Credentials and DOI prefix (10.eeelection) issued for production use.

## Endpoint

Production: https://api.datacite.org/dois
Sandbox: https://api.stage.datacite.org/dois

## Protocol and Authentication

REST over HTTPS with JSON payloads. BasicAuth (username/password) or Bearer token authentication.

**BasicAuth:**
Username: DataCite account username
Password: API token (preferred over account password)

**Bearer Token:**
Authorization: Bearer {api_token}

## Request and Response Shape

**DOI Create/Mint:**
Request: `POST /dois {prefix: "10.eeelection", suffix: "2027.001", url: "https://eel.iebc.or.ke/bundles/2027/...", titles: [{title: "Kenya 2027 Election Results Bundle", lang: "en"}], creators: [{name: "IEBC", affiliation: "Independent Electoral and Boundaries Commission"}], publicationYear: 2027, resourceType: {resourceType: "Dataset", resourceTypeGeneral: "Dataset"}}`
Response: `{id: "10.eeelection/2027.001", type: "dois", attributes: {doi: "10.eeelection/2027.001", url: "https://...", state: "draft|registered|findable"}}`

**Metadata Update:**
Request: `PUT /dois/10.eeelection/2027.001 {attributes: {titles: [...], descriptions: [...], subjects: [...]}}`
Response: `{id: "10.eeelection/2027.001", attributes: {state: "findable", updated: "2024-08-21T..."}}`

**DOI Resolve:**
Request: `GET /dois/10.eeelection/2027.001`
Response: `{id: "10.eeelection/2027.001", attributes: {doi: "...", url: "...", titles: [...], creators: [...], publicationYear: 2027, state: "findable", updated: "..."}}`

## Sample Curl

```bash
curl -u "$DATACITE_USERNAME:$DATACITE_PASSWORD" \
  -H "Content-Type: application/vnd.api+json" \
  -d '{"data":{"type":"dois","attributes":{"prefix":"10.eeelection","suffix":"2027.001","url":"https://eel.iebc.or.ke/bundles/2027/results.zip","titles":[{"title":"Kenya 2027 Election Results Bundle"}],"creators":[{"name":"IEBC"}],"publicationYear":2027,"resourceType":{"resourceType":"Dataset","resourceTypeGeneral":"Dataset"},"state":"registered"}}}' \
  https://api.datacite.org/dois

curl https://api.datacite.org/dois/10.eeelection/2027.001
```

## Health Probe

GET /dois returns 200 if DataCite is reachable (no auth required for listing).

## Rate Limits

100 requests/minute per API token. DOI minting typically takes 1-2 seconds, metadata updates immediate.

## PII Enforcement

DOI metadata (titles, creators, publication dates) is public by design. No sensitive or restricted information stored in DOI records. All metadata is searchable and citable by the public.

## Chaos Toggle

DATACITE_CHAOS=true enables chaos injection. Causes random DOI minting failures and metadata update rejections (test only).

## Fallback Behavior

On DataCite unavailability, DOI intent recorded locally. Manual minting workflow available: operator defers minting to next maintenance window (weekly). Local record retains intended DOI prefix and suffix for later registration.

## Circuit Breaker Policy

Threshold: 3 consecutive failures
Open duration: 30 seconds
Rationale: DataCite has high availability; transient failures recover quickly
Half-open: permits 1 probe request

## Environment Variables

DATACITE_BASE_URL: DataCite API base URL
DATACITE_USERNAME: DataCite account username
DATACITE_PASSWORD: DataCite API token (for Basic auth password field)
DATACITE_PREFIX: DOI prefix assigned to commission (10.eeelection)
DATACITE_TIMEOUT_SEC: Request timeout in seconds (default: 30)
DATACITE_CHAOS: "true" enables chaos (test only)

## Related Modules

M69 (long-term-archive DOI minting), M14 (reproducibility-bundle DOI registration)

## Runbook

docs/runbooks/datacite-doi-management.md (DOI minting workflow, metadata updates, public resolution, fallback procedures)

