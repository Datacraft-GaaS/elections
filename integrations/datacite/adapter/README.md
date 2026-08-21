# DataCite Adapter

Production adapter for DataCite international DOI registration API. Handles DOI minting, metadata updates, and public DOI resolution for election datasets and reproducibility bundles.

## Purpose

Bridges eel core system (M69, M14 modules) to DataCite API. Manages BasicAuth credentials, DOI prefix/suffix validation, metadata JSON-API formatting, DOI state tracking, and public resolution URLs.

## Layout

- `service.py`: Main adapter service. Handles DOI minting, metadata updates, DOI state queries, public URL registration, suffix generation.
- `views.py`: Pydantic models for DataCite JSON-API payloads. Models for DOICreate, MetadataUpdate, DOIResolve operations. Resource type and state enums.
- `client.py`: HTTP client initialization with BasicAuth and timeout configuration.
- `suffix_generator.py`: DOI suffix generation (typically year + sequence number).
- `__init__.py`: Package entry point, exports main service class.

## Key Classes and Functions

**DataciteService** (async):
- `mint_doi(title, url, creators, year)`: Register new DOI, returns DOIRecord.
- `update_metadata(doi, metadata)`: Update existing DOI metadata, returns UpdateReceipt.
- `resolve_doi(doi)`: Fetch DOI metadata from public API, returns DOIData.
- `generate_suffix()`: Generate next DOI suffix (e.g., 2027.001).

**Models** (in views.py):
- `DOICreateRequest`: Title, URL, creators list, publication year, resource type.
- `DOICreateResponse`: DOI string, state (draft, registered, findable), creation timestamp.
- `DOIUpdateRequest`: Metadata fields to update (titles, descriptions, subjects).
- `DOIMetadata`: Titles, creators, subjects, descriptions, publication year, resource type.
- `DOIData`: DOI string, URL, state, creation date, update date, complete metadata.

## Environment Variables

DATACITE_BASE_URL: DataCite API base URL
DATACITE_USERNAME: DataCite account username
DATACITE_PASSWORD: DataCite API token
DATACITE_PREFIX: DOI prefix (e.g., 10.eeelection)
DATACITE_TIMEOUT_SEC: Request timeout in seconds (default: 30)
DATACITE_CHAOS: "true" enables chaos injection (test only)

## Build and Test

Build package:
```bash
cd adapter
uv build
```

Install in development:
```bash
uv pip install -e .
```

Type check:
```bash
uv run pyright src/
```

Run tests:
```bash
uv run pytest tests/ -vxs
```
