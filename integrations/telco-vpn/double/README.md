# telco-vpn/double

Canned-response mock for VPN tunnel health monitoring. Simulates IPsec and WireGuard.

## Running the mock

```bash
python -m eel_double_telco_vpn --port 8084
```

Listens on localhost:8084. Responds to tunnel status queries with synthetic data.

## Chaos knobs

Environment variables for fault injection:

- VPN_DOUBLE_CHAOS: none | tunnel_down | packet_loss | latency (default: none)
  - tunnel_down: mark primary tunnel as down, secondary up
  - packet_loss: simulate 5% packet loss
  - latency: add 500ms latency to both tunnels

- VPN_TUNNELS_UP: comma-separated tunnel names (default: ipsec_safaricom,wireguard)

- VPN_DOUBLE_REKEY_INTERVAL_SECS: interval for simulating rekey events (default: 3600)

## Canned responses

```
{
  "tunnels": [
    {"name": "ipsec_safaricom", "state": "up", "packets": 1234567, "bytes": 987654321},
    {"name": "wireguard", "state": "up", "packets": 5678901, "bytes": 234567890}
  ]
}
```

If VPN_DOUBLE_CHAOS=tunnel_down:
```
{"name": "ipsec_safaricom", "state": "down", "packets": 0, "bytes": 0}
```
