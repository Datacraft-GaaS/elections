# BagIt Packaging

BagIt packaging service for long-term digital preservation of election artifacts. Assembles archival BagIt packages from election data (results, forms, audit logs), generates preservation attestations, and submits to KNADS and DataCite for DOI registration.

## Overview

Module: M69 (Long-Term Archive)

Digital preservation service following BagIt standard (RFC 8493) for reliable long-term archival. Produces cryptographically signed archive packages and registers them in national digital repository (KNADS) and international registry (DataCite).

## FastAPI Endpoints

- POST `/bagit/pack` - Create BagIt package for collection
- GET `/bagit/{bag_id}/status` - Poll packaging status
- POST `/bagit/{bag_id}/attest` - Sign preservation attestation
- GET `/health` - Service health check

## Pydantic Models

- `ArchiveCollection` - Collection metadata (id, election_id, artifact_types: list[str])
- `IngestJob` - Ingest job (id, collection_id, status, knads_ref, doi, created_at)
- `PreservationAttestation` - Signed attestation (bag_id, attester_id, hash_algorithm, checksum)

## Environment Variables

- `DATABASE_URL` - PostgreSQL connection string
- `KAFKA_BROKERS` - Kafka broker endpoints
- `KNADS_ADAPTER_URL` - KNADS (Kenya National Archive Digital Services) API
- `DATACITE_ADAPTER_URL` - DataCite API for DOI registration
- `S3_BUCKET` - Archival storage bucket
- `BAGIT_PROFILE_VERSION` - BagIt profile version (default: 0.97)

## Build & Test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Production Run

```bash
python -m bagit.worker
```

## BagIt Contents

- `bagit.txt` - BagIt version and encoding declaration
- `manifest-sha512.txt` - Cryptographic checksums for all files
- `tagmanifest-sha512.txt` - Checksums of metadata files
- `bag-info.txt` - Dublin Core metadata (title, creator, date, etc.)
- `data/` - Election artifact files (results, forms, audit logs)

## Notes

- All bags are checksummed with SHA-512
- Preservation attestations are digitally signed
- DOI registration enables long-term citability (100-year archives)
- KNADS submission creates backup copy in national repository
- Packaging time: ~10 minutes per 1GB of artifacts
