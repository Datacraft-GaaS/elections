# integrations/ipfs

Kubo API and libp2p for decentralized document storage and audit trail preservation.

**MoU Status: Open source integration (no MoU required)**

## Module usage

- M95 (audit-trail-immutable) - store election audit logs and evidence on IPFS
- M103 (evidence-archival) - preserve election materials and court documents
- M112 (integrity-verification) - verify document integrity using IPFS content hashing

## Wire protocol and authentication

Protocol: REST (HTTPS to Kubo daemon)
Authentication: peer-key (multiaddr identity, optional HTTP Basic auth)
Endpoints: Kubo HTTP API (self-hosted or public gateway)

### Endpoints

| Path | Method | Description |
|---|---|---|
| `/api/v0/add` | POST | Add file/directory to IPFS |
| `/api/v0/get` | GET | Retrieve file by CID (content hash) |
| `/api/v0/cat` | GET | Stream file contents |
| `/api/v0/dag/get` | GET | Get DAG node by CID |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| contentHash | C (Public) | IPFS CID (immutable content reference) |
| fileMetadata | C (Public) | Size, MIME type, upload timestamp |
| documentContent | B (Internal) | Stored document (inherits source classification) |
| accessLog | C (Public) | Retrieval statistics (aggregated) |

## Request and response shape

**Add File Request (multipart/form-data):**
- file: binary (required, up to 256MB per file)
- progress: boolean (report progress)
- wrap-with-directory: boolean (add CIDv1 wrapper)

**Add File Response:**
- Name: string (filename)
- Hash: string (IPFS CID, base32)
- Size: integer (bytes)

**Get File Request:**
- arg: string (CID, required)
- compression: string (optional, gzip/brotli)

**Get File Response:**
- binary stream (Content-Type based on stored MIME type)

## Sample usage

```bash
# Add document to IPFS
curl -F file=@ballot.pdf \
  -H "Authorization: Bearer $IPFS_TOKEN" \
  http://localhost:5001/api/v0/add?progress=true

# Response: {"Name":"ballot.pdf","Hash":"QmYjtig7...","Size":"245123"}

# Retrieve document by CID (works with any IPFS node)
curl http://ipfs.example.com/ipfs/QmYjtig7... > ballot.pdf

# Stream file contents
curl http://localhost:5001/api/v0/cat?arg=QmYjtig7...
```

## Health and readiness

```bash
# Check Kubo daemon status
curl http://localhost:5001/api/v0/id
# Response: {"ID":"QmXxxx...","PublicKey":"...","Addresses":["..."]}

# Check peers and network connectivity
curl http://localhost:5001/api/v0/swarm/peers
```

## Rate limits

Unlimited local requests (self-hosted node). Public gateways enforce rate limits.

| Service | Metric | Limit | Window |
|---|---|---|---|
| Local Kubo | File adds | unlimited | - |
| Local Kubo | File retrieval | unlimited | - |
| Public gateway | HTTP requests | 100/min | rolling 1-minute (per IP) |
| Public gateway | Bandwidth | 1GB/day | calendar day (per IP) |

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 3 consecutive errors (connection refused, timeout, 5xx) |
| open duration | 30 seconds |
| retry policy | exponential backoff (base 200ms, max 5s) |

## Chaos testing

Set `IPFS_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | return 504 after 25s |
| error | return 503 Service Unavailable |
| hash_mismatch | return corrupted file |

## Fallback strategy

Circuit open per node: failover to secondary IPFS node. If unavailable, queue document storage request (use local filesystem staging). Retry adds periodically (exponential backoff). For retrieval, use cached local copy (TTL: 7 days) or fall back to read-only public gateway. Queue TTL: 30 days.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| IPFS_HOST | yes | localhost | Kubo daemon hostname |
| IPFS_PORT | yes | 5001 | Kubo HTTP API port |
| IPFS_GATEWAY_URL | yes | - | Public IPFS gateway for retrieval |
| IPFS_AUTH_TOKEN | no | - | Bearer token for restricted nodes |
| IPFS_TIMEOUT_SECS | no | 25 | Request timeout |
| IPFS_MAX_FILE_SIZE_MB | no | 256 | Maximum file size |
| IPFS_CHAOS | no | false | Chaos mode (timeout/error/hash_mismatch) |

## Related modules

- M95 (audit-trail-immutable) - primary consumer
- M103 (evidence-archival) - long-term storage provider
- M112 (integrity-verification) - hash validation

## Runbook

Refer to `docs/runbooks/integrations/ipfs.md` for node failover procedures, DAG pin management, and public gateway performance tuning.
