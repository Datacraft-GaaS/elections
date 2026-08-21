# integrations/telco-vpn

I03 Telco Leased Line / WireGuard VPN overlay for kit network transport.

## Purpose

Provides secure, always-on IP connectivity from field kits to backend services. Combines operator IPsec leased line (primary) with WireGuard overlay (fallback). Handles site-to-site VPN roaming (device mobility).

## Modules using this adapter

P01 (kit network layer), P04 (offline kit queuing), all kit applications.

## Protocol

IPsec (IKEv2, AES-256-GCM) + WireGuard overlay. Dual-tunnel for redundancy: Safaricom primary, Airtel secondary.

## Authentication

Pre-shared key (PSK) per site + x.509 certificates per VPN endpoint (site CA issued).

## PII enforcement

None (transport layer). Payloads encrypted end-to-end by consuming services.

## Rate limits

Bandwidth-limited per site SLA. Typical: 4 Mbps sustained, 10 Mbps burst per kit.

## Circuit breaker

Always-on. WireGuard roaming handles multi-network handoff automatically. No failover delay.

## Fallback

If both IPsec and WireGuard unreachable for 5+ minutes, escalate to satellite fallback (I14).

## Chaos toggle

VPN_CHAOS_ENABLED=true (double only). Simulates tunnel down, packet loss, latency spike.

## Health probe

GET /vpn/tunnels -> list of tunnel statuses (up, down, rekeying).

## MoU status

Safaricom/Airtel leased line contracts signed. WireGuard config per site on operations checklist.
