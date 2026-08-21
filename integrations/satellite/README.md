# integrations/satellite

UDP transport and REST management for satellite connectivity via VSAT (Very Small Aperture Terminal) and LEO (Low Earth Orbit) satellite services.

**MoU Status: Safaricom VSAT MoU signed; Starlink LEO agreement in place**

## Module usage

- Connectivity for remote polling stations and county offices
- Election result transmission from areas with limited terrestrial coverage
- Real-time ballot tracking in rural and underserved regions

## Wire protocol and authentication

Protocol: UDP (transport) + REST (HTTPS management)
Authentication: Pre-shared key (PSK) for gateway access
Endpoints: Satellite gateway management API

### Endpoints

| Path | Method | Description |
|---|---|---|
| `/gateway/status` | GET | Check satellite link status |
| `/link/test` | POST | Test link connectivity and latency |
| `/queue/send` | POST | Queue data transmission via satellite |
| `/queue/{queueId}` | GET | Check transmission status |
| `/statistics` | GET | Get bandwidth and usage statistics |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| gatewayID | C (Public) | Satellite gateway identifier |
| linkStatus | C (Public) | Connection state (no PII) |
| bandwidth | C (Public) | Usage metrics (non-sensitive) |
| latency | C (Public) | Ping times (no PII) |
| timestamp | C (Public) | Metadata timestamp |

## Request and response shape

**Link Test Request:**
- dataSize: integer (bytes to test, default 1024)
- targetGateway: string (required, gateway ID or name)
- priority: enum (normal, high, critical)

**Link Test Response:**
- latency: integer (milliseconds)
- packetLoss: float (0.0-1.0, percentage)
- bandwidth: integer (bps)
- status: enum (connected, degraded, offline)
- estimatedSendTime: integer (seconds for full payload)

**Queue Send Request:**
- data: string or base64 (payload to transmit)
- destination: string (polling station ID or coordinates)
- priority: enum (normal, high, critical)
- requireAck: boolean (request confirmation)

**Queue Send Response:**
- queueId: string (UUID)
- status: enum (queued, transmitting, sent, failed)
- estimatedDelivery: ISO 8601 timestamp
- attempt: integer (retry attempt number)

## Sample usage

```bash
# Test satellite link (requires PSK)
curl -X POST https://gateway.sat.example.com/link/test \
  -H "Authorization: Bearer $SATELLITE_PSK" \
  -H "Content-Type: application/json" \
  -d '{
    "dataSize": 10240,
    "targetGateway": "VSAT-NAIROBI-01",
    "priority": "high"
  }'

# Queue data for transmission
curl -X POST https://gateway.sat.example.com/queue/send \
  -H "Authorization: Bearer $SATELLITE_PSK" \
  -H "Content-Type: application/json" \
  -d '{
    "data": "base64_encoded_election_results",
    "destination": "POLL-STATION-015",
    "priority": "critical",
    "requireAck": true
  }'
```

## Health and readiness

```bash
curl -s -H "Authorization: Bearer $SATELLITE_PSK" \
  https://gateway.sat.example.com/gateway/status
# Response: {"status": "connected", "provider": "starlink", "latency": 45}
```

## Rate limits

Bandwidth-limited by satellite service tier:

| Service | Downstream | Upstream | Concurrent |
|---|---|---|---|
| VSAT | 1 Mbps | 1 Mbps | 4 connections |
| LEO (Starlink) | 50-100 Mbps | 50-100 Mbps | unlimited |

Queue limits: 1000 items per polling station per day

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 3 timeout errors (5+ min no response) |
| open duration | 5 minutes (satellite lag tolerance) |
| retry policy | exponential backoff (base 500ms, max 30s) |

Note: Extended timeout due to satellite latency (500ms-1.5s nominal).

## Chaos testing

Set `SATELLITE_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | no response for 5+ minutes |
| error | return 503 Service Unavailable |
| slow | add 2s additional latency |
| packet_loss | simulate 10-30% packet loss |

## Fallback strategy

Circuit open: queue transmissions to local storage. Retry every 15 minutes with exponential backoff. Fall back to terrestrial network (4G/5G) when available. For critical data (election results), use SMS or voice channel as last resort. Queue TTL: 7 days.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| SATELLITE_GATEWAY_HOST | yes | - | Satellite gateway hostname |
| SATELLITE_GATEWAY_PORT | yes | - | Gateway port (typically 443 for REST) |
| SATELLITE_PSK | yes | - | Pre-shared key for authentication |
| SATELLITE_UDP_PORT | no | 5060 | UDP port for data transport |
| SATELLITE_CHAOS | no | false | Chaos mode (timeout/error/slow) |
| SATELLITE_TIMEOUT_SECS | no | 300 | Request timeout (5 minutes) |
| SATELLITE_QUEUE_PATH | no | ./var/sat-queue | Offline queue storage path |

## Related modules

- Remote polling site communications module - primary consumer
- Election result transmission module - data sender
- Backup connectivity module - failover provider

## Runbook

Refer to `docs/runbooks/integrations/satellite.md` for gateway failover procedures, PSK rotation, and offline queue management.
