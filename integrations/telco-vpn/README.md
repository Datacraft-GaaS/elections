# integrations/telco-vpn

I03 IPsec tunnel health probe via strongSwan swanctl.

## Endpoints

- `GET /vpn/tunnels` - JSON list of tunnel statuses
- `GET /metrics` - Prometheus metrics: `eel_vpn_tunnel_up{carrier=safaricom|airtel|telkom}`
- `GET /health`

## Wire protocol

Shells out to `swanctl --list-sas --raw`. Parses CHILD_SA INSTALLED state.

## Double knobs

| Env var | Default |
|---|---|
| VPN_TUNNELS_UP | safaricom,airtel,telkom |

## SLA

Sub-second response (local swanctl call). Hard timeout 5 s.
