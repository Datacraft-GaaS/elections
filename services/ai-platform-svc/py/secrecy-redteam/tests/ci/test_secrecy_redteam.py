from __future__ import annotations

import asyncio
from datetime import datetime, timezone

from httpx import ASGITransport, AsyncClient

from secrecy_redteam.service import (
    app,
    attestations_store,
    engagements_store,
    findings_store,
    remediations_store,
)

BASE = "http://test"


def _client() -> AsyncClient:
    return AsyncClient(transport=ASGITransport(app=app), base_url=BASE)


def _now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def setup_function(_: object) -> None:
    engagements_store.clear()
    findings_store.clear()
    remediations_store.clear()
    attestations_store.clear()


def test_health() -> None:
    async def _run() -> None:
        async with _client() as c:
            r = await c.get("/v1/redteam/health")
        assert r.status_code == 200
        assert r.json() == {"ok": True}

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_create_engagement() -> None:
    async def _run() -> None:
        payload = {
            "title": "Q3 Red Team Op",
            "scope": "auth subsystem",
            "status": "active",
            "started_at": _now_iso(),
        }
        async with _client() as c:
            r = await c.post("/v1/redteam/engagements", json=payload)
        assert r.status_code == 201
        body = r.json()
        assert body["title"] == "Q3 Red Team Op"
        assert body["status"] == "active"
        assert body["engagement_id"]

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_add_finding_to_engagement() -> None:
    async def _run() -> None:
        async with _client() as c:
            re = await c.post(
                "/v1/redteam/engagements",
                json={
                    "title": "Op Alpha",
                    "scope": "api",
                    "status": "active",
                    "started_at": _now_iso(),
                },
            )
        eid = re.json()["engagement_id"]

        finding_payload = {
            "engagement_id": eid,
            "title": "SQL injection in login",
            "severity": "critical",
            "description": "Unparameterised query allows full DB dump",
            "cve_ids": [],
            "discovered_at": _now_iso(),
        }
        async with _client() as c:
            rf = await c.post("/v1/redteam/findings", json=finding_payload)
        assert rf.status_code == 201
        body = rf.json()
        assert body["severity"] == "critical"
        assert body["engagement_id"] == eid

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_remediation_workflow() -> None:
    async def _run() -> None:
        async with _client() as c:
            re = await c.post(
                "/v1/redteam/engagements",
                json={
                    "title": "Op Beta",
                    "scope": "web",
                    "status": "active",
                    "started_at": _now_iso(),
                },
            )
        eid = re.json()["engagement_id"]

        async with _client() as c:
            rf = await c.post(
                "/v1/redteam/findings",
                json={
                    "engagement_id": eid,
                    "title": "XSS in admin panel",
                    "severity": "high",
                    "description": "Reflected XSS in search",
                    "discovered_at": _now_iso(),
                },
            )
        fid = rf.json()["finding_id"]

        async with _client() as c:
            rr = await c.post(
                "/v1/redteam/remediations",
                json={
                    "finding_id": fid,
                    "status": "open",
                    "owner": "dev_team",
                    "due_date": _now_iso(),
                },
            )
        assert rr.status_code == 201
        rid = rr.json()["remediation_id"]
        assert rr.json()["status"] == "open"

        async with _client() as c:
            ru = await c.patch(f"/v1/redteam/remediations/{rid}", json={"status": "in_progress"})
        assert ru.json()["status"] == "in_progress"

        async with _client() as c:
            rv = await c.patch(f"/v1/redteam/remediations/{rid}", json={"status": "resolved"})
        assert rv.json()["status"] == "resolved"
        assert rv.json()["resolved_at"] is not None

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())


def test_iab_attestation() -> None:
    async def _run() -> None:
        async with _client() as c:
            re = await c.post(
                "/v1/redteam/engagements",
                json={
                    "title": "Op Gamma",
                    "scope": "infra",
                    "status": "active",
                    "started_at": _now_iso(),
                },
            )
        eid = re.json()["engagement_id"]

        att_payload = {
            "engagement_id": eid,
            "attested_by": "ciso@example.com",
            "staging_clone_id": "clone-abc-123",
            "attestation_text": "I attest that staging clone is isolated",
            "attested_at": _now_iso(),
        }
        async with _client() as c:
            ra = await c.post("/v1/redteam/attestations", json=att_payload)
        assert ra.status_code == 201
        body = ra.json()
        assert body["attested_by"] == "ciso@example.com"
        assert body["engagement_id"] == eid

    loop = asyncio.get_event_loop()
    loop.run_until_complete(_run())
