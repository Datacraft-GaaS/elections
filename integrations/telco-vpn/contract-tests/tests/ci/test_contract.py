"""VPN adapter contract tests against the double."""

from __future__ import annotations
import os
import httpx
import pytest

DOUBLE_URL = os.environ.get("VPN_DOUBLE_URL", "http://localhost:8090")


@pytest.mark.asyncio
async def test_tunnels_list() -> None:
    async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
        resp = await client.get("/vpn/tunnels")
    assert resp.status_code == 200
    data = resp.json()
    assert isinstance(data, list)
    carriers = {t["carrier"] for t in data}
    assert carriers == {"safaricom", "airtel", "telkom"}


@pytest.mark.asyncio
async def test_metrics_prometheus() -> None:
    async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
        resp = await client.get("/metrics")
    assert resp.status_code == 200
    text = resp.text
    assert "eel_vpn_tunnel_up" in text
    assert 'carrier="safaricom"' in text


@pytest.mark.asyncio
async def test_health() -> None:
    async with httpx.AsyncClient(base_url=DOUBLE_URL) as client:
        resp = await client.get("/health")
    assert resp.status_code == 200
