"""VPN double: mock swanctl output via HTTP endpoints matching the real adapter."""

from __future__ import annotations

import os

from fastapi import FastAPI

_TUNNELS_UP = os.environ.get("VPN_TUNNELS_UP", "safaricom,airtel,telkom").split(",")

app = FastAPI(title="eel-adapter-telco-vpn-double")


@app.get("/vpn/tunnels")
async def tunnels() -> list[dict]:
    carriers = ["safaricom", "airtel", "telkom"]
    return [
        {
            "carrier": c,
            "name": f"eel-{c}",
            "up": c in _TUNNELS_UP,
            "bytes_in": 0,
            "bytes_out": 0,
        }
        for c in carriers
    ]


@app.get("/metrics")
async def metrics() -> str:
    carriers = ["safaricom", "airtel", "telkom"]
    lines = [
        f'eel_vpn_tunnel_up{{carrier="{c}"}} {1 if c in _TUNNELS_UP else 0}'
        for c in carriers
    ]
    return "\n".join(lines)


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok"}
