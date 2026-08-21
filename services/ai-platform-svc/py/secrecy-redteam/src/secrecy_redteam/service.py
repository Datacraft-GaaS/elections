from __future__ import annotations

import logging
from datetime import datetime, timezone
from typing import Any

from fastapi import FastAPI, HTTPException
from uuid6 import uuid7

from secrecy_redteam.views import Engagement, Finding, IabAttestation, Remediation

logger = logging.getLogger(__name__)

app = FastAPI(title="eel-secrecy-redteam", version="0.1.0")

engagements_store: dict[str, Engagement] = {}
findings_store: dict[str, Finding] = {}
remediations_store: dict[str, Remediation] = {}
attestations_store: dict[str, IabAttestation] = {}


def _uuid7str() -> str:
    return str(uuid7())


def _now() -> datetime:
    return datetime.now(timezone.utc)


def _log_create(kind: str, id_: str) -> None:
    logger.info("eel.redteam.%s.created.v1 id=%s", kind, id_)


@app.get("/v1/redteam/health")
async def health() -> dict[str, bool]:
    return {"ok": True}


# Engagements
@app.post("/v1/redteam/engagements", status_code=201)
async def create_engagement(body: dict[str, Any]) -> Engagement:
    eid = _uuid7str()
    eng = Engagement(
        engagement_id=eid,
        title=body["title"],
        scope=body["scope"],
        status=body.get("status", "planned"),
        started_at=body.get("started_at", _now()),
    )
    engagements_store[eid] = eng
    _log_create("engagement", eid)
    return eng


@app.get("/v1/redteam/engagements")
async def list_engagements() -> list[Engagement]:
    return list(engagements_store.values())


@app.get("/v1/redteam/engagements/{engagement_id}")
async def get_engagement(engagement_id: str) -> Engagement:
    eng = engagements_store.get(engagement_id)
    if eng is None:
        raise HTTPException(status_code=404, detail="engagement not found")
    return eng


# Findings
@app.post("/v1/redteam/findings", status_code=201)
async def create_finding(body: dict[str, Any]) -> Finding:
    if body["engagement_id"] not in engagements_store:
        raise HTTPException(status_code=404, detail="engagement not found")
    fid = _uuid7str()
    finding = Finding(
        finding_id=fid,
        engagement_id=body["engagement_id"],
        title=body["title"],
        severity=body["severity"],
        description=body["description"],
        cve_ids=body.get("cve_ids", []),
        discovered_at=body.get("discovered_at", _now()),
    )
    findings_store[fid] = finding
    _log_create("finding", fid)
    return finding


@app.get("/v1/redteam/findings")
async def list_findings() -> list[Finding]:
    return list(findings_store.values())


# Remediations
@app.post("/v1/redteam/remediations", status_code=201)
async def create_remediation(body: dict[str, Any]) -> Remediation:
    if body["finding_id"] not in findings_store:
        raise HTTPException(status_code=404, detail="finding not found")
    rid = _uuid7str()
    rem = Remediation(
        remediation_id=rid,
        finding_id=body["finding_id"],
        status=body.get("status", "open"),
        owner=body["owner"],
        due_date=body["due_date"],
    )
    remediations_store[rid] = rem
    _log_create("remediation", rid)
    return rem


@app.patch("/v1/redteam/remediations/{remediation_id}")
async def update_remediation(remediation_id: str, body: dict[str, Any]) -> Remediation:
    rem = remediations_store.get(remediation_id)
    if rem is None:
        raise HTTPException(status_code=404, detail="remediation not found")
    update: dict[str, Any] = {}
    if "status" in body:
        update["status"] = body["status"]
    if body.get("status") == "resolved":
        update["resolved_at"] = _now()
    updated = rem.model_copy(update=update)
    remediations_store[remediation_id] = updated
    return updated


@app.get("/v1/redteam/remediations")
async def list_remediations() -> list[Remediation]:
    return list(remediations_store.values())


# IAB Attestations
@app.post("/v1/redteam/attestations", status_code=201)
async def create_attestation(body: dict[str, Any]) -> IabAttestation:
    if body["engagement_id"] not in engagements_store:
        raise HTTPException(status_code=404, detail="engagement not found")
    aid = _uuid7str()
    att = IabAttestation(
        attestation_id=aid,
        engagement_id=body["engagement_id"],
        attested_by=body["attested_by"],
        staging_clone_id=body["staging_clone_id"],
        attestation_text=body["attestation_text"],
        attested_at=body.get("attested_at", _now()),
    )
    attestations_store[aid] = att
    _log_create("attestation", aid)
    return att


@app.get("/v1/redteam/attestations")
async def list_attestations() -> list[IabAttestation]:
    return list(attestations_store.values())
