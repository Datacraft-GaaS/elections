# DataCite Test Double

Mock REST server simulating DataCite international DOI registration API for integration testing. Accepts DOI minting requests, simulates metadata management, and supports chaos injection for failure scenario testing.

## Purpose

Provides deterministic test environment without DataCite dependency. Validates adapter BasicAuth credentials, JSON-API payload formatting, DOI suffix generation, metadata updates, and public resolution. Used by contract-tests and development workflows.

## How to Run

Start the double server:
```bash
cd double
uv run python -m eel_double_datacite
```

Server listens on http://localhost:8009 by default.

To override port:
```bash
PORT=9009 uv run python -m eel_double_datacite
```

Environment variable: DATACITE_DOUBLE_PORT (default: 8009).

## Chaos Knobs

Control failure modes via environment variables:

DATACITE_CHAOS_ENABLED: "true" to activate all chaos (default: false)
DATACITE_CHAOS_MINT_FAILURE_RATE: Fraction [0.0-1.0] of DOI minting returning 400 (default: 0.0)
DATACITE_CHAOS_UPDATE_FAILURE_RATE: Fraction [0.0-1.0] of metadata updates failing (default: 0.0)
DATACITE_CHAOS_LATENCY_MS: Milliseconds to delay responses (default: 0)
DATACITE_CHAOS_TIMEOUT_SEC: Force request timeout after N seconds (default: 0, disabled)

Example:
```bash
DATACITE_CHAOS_ENABLED=true DATACITE_CHAOS_MINT_FAILURE_RATE=0.1 uv run python -m eel_double_datacite
```

## Canned Responses

**DOI Mint** (POST /dois):
Returns `{"data":{"id":"10.eeelection/2027.001","type":"dois","attributes":{"doi":"10.eeelection/2027.001","url":"https://eel.iebc.or.ke/bundles/2027/results.zip","state":"registered","created":"2024-08-21T10:30:00Z"}}}`.
On missing required field: returns 400 `{"errors":[{"title":"Invalid request"}]}`.
On chaos: returns 400 failure at configured rate.

**Metadata Update** (PUT /dois/10.eeelection/2027.001):
Returns `{"data":{"id":"10.eeelection/2027.001","attributes":{"state":"findable","updated":"2024-08-21T11:00:00Z"}}}`.
On invalid DOI format: returns 400 `{"errors":[{"title":"Invalid DOI"}]}`.

**DOI Resolve** (GET /dois/10.eeelection/2027.001):
Returns `{"data":{"id":"10.eeelection/2027.001","type":"dois","attributes":{"doi":"10.eeelection/2027.001","url":"https://...","titles":[{"title":"Kenya 2027 Election Results"}],"creators":[{"name":"IEBC"}],"publicationYear":2027,"state":"findable"}}}`.
On missing DOI: returns 404 `{"errors":[{"title":"DOI not found"}]}`.

**List DOIs** (GET /dois):
Returns empty list (public endpoint, no auth required): `{"data":[]}`.

## Endpoints Exposed

GET /dois - List DOIs (public, no auth)
POST /dois - Mint new DOI
GET /dois/{doi} - Resolve DOI metadata (public, no auth)
PUT /dois/{doi} - Update DOI metadata
