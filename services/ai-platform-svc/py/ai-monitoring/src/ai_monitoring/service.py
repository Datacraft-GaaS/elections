from __future__ import annotations

import logging
from datetime import datetime, timezone
from typing import Any

from fastapi import FastAPI, HTTPException
from uuid6 import uuid7

from ai_monitoring.views import (
    DetectionReportStatus,
    ModelDecisionSummary,
    PiiStripStats,
    WeeklyDetectionReport,
)

logger = logging.getLogger(__name__)

app = FastAPI(title="eel-ai-monitoring", version="0.1.0")

reports_store: dict[str, WeeklyDetectionReport] = {}

now = lambda: datetime.now(timezone.utc)


def uuid7str() -> str:
    return str(uuid7())


def _log_publish(report_id: str) -> None:
    logger.info("eel.ai.detection_report_published.v1 report_id=%s", report_id)


@app.get("/v1/ai/monitoring/health")
async def health() -> dict[str, bool]:
    return {"ok": True}


@app.get("/v1/ai/monitoring/reports")
async def list_reports() -> list[WeeklyDetectionReport]:
    return list(reports_store.values())


@app.get("/v1/ai/monitoring/reports/{report_id}")
async def get_report(report_id: str) -> WeeklyDetectionReport:
    report = reports_store.get(report_id)
    if report is None:
        raise HTTPException(status_code=404, detail="report not found")
    return report


@app.post("/v1/ai/monitoring/reports", status_code=201)
async def create_report(body: dict[str, Any]) -> WeeklyDetectionReport:
    report_id = uuid7str()
    report = WeeklyDetectionReport(
        id=report_id,
        report_week=body.get("report_week", ""),
        layer_summaries=body.get("layer_summaries", {}),
        pii_strip_stats=body.get("pii_strip_stats", {}),
        published_at=None,
        tlog_entry=None,
        status=DetectionReportStatus.PENDING,
    )
    reports_store[report_id] = report
    return report


@app.post("/v1/ai/monitoring/reports/{report_id}/publish")
async def publish_report(report_id: str) -> WeeklyDetectionReport:
    report = reports_store.get(report_id)
    if report is None:
        raise HTTPException(status_code=404, detail="report not found")
    updated = report.model_copy(
        update={
            "published_at": now(),
            "status": DetectionReportStatus.PUBLISHED,
        }
    )
    reports_store[report_id] = updated
    _log_publish(report_id)
    return updated


@app.get("/v1/ai/monitoring/decision-summary")
async def decision_summary() -> list[ModelDecisionSummary]:
    return []


@app.get("/v1/ai/monitoring/pii-stats")
async def pii_stats() -> PiiStripStats:
    epoch = datetime(1970, 1, 1, tzinfo=timezone.utc)
    return PiiStripStats(
        total_calls=0,
        total_tokens_replaced=0,
        bypass_count=0,
        period_start=epoch,
        period_end=epoch,
    )
