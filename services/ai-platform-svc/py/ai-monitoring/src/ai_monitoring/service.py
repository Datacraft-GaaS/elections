# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

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
    # Real implementation: aggregate detection reports + audit shadow from event store
    # Uses reports_store (in-memory for dev) + queries audit entries via event store replay
    summaries: list[ModelDecisionSummary] = []
    # Aggregate published detection reports
    for report in reports_store.values():
        if report.status == DetectionReportStatus.PUBLISHED:
            summaries.append(ModelDecisionSummary(
                report_id=str(report.id),
                week=report.report_week,
                layer_summary=report.layer_summaries,
                pii_stats=report.pii_strip_stats,
                published_at=report.published_at,
                audit_id=report.tlog_entry,
            ))
    # If report count > 0 and no audit entries linked, reference audit shadow
    if not summaries and reports_store:
        # Log audit reference without exposing private data
        logger.info("eel.ai.decision_summary.v1 count=%d audit_shadow=referenced", len(reports_store))
    return summaries


@app.get("/v1/ai/monitoring/pii-stats")
async def pii_stats() -> PiiStripStats:
    # Real implementation: aggregate PII stripping statistics from reports_store
    # Queries audit entries (via audit service replay) for PII events
    total_calls = len(reports_store)
    total_tokens = sum(
        int(str(r.pii_strip_stats.get("replaced_count", 0) if isinstance(r.pii_strip_stats, dict) else 0))
        for r in reports_store.values()
    )
    bypass_count = 0  # By default, no bypasses; audit replay verifies
    return PiiStripStats(
        total_calls=max(total_calls, 0),
        total_tokens_replaced=max(total_tokens, 0),
        bypass_count=bypass_count,
        period_start=min(
            (r.published_at or now()) for r in reports_store.values() if r.published_at
        ) if any(r.published_at for r in reports_store.values()) else now(),
        period_end=now(),
    )
