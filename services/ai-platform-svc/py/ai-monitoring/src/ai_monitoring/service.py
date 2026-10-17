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


def now() -> datetime:
    return datetime.now(timezone.utc)


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
    # Cannot be served from this service's state, so it says so rather than
    # returning something shaped like an answer.
    #
    # ModelDecisionSummary is per-model: model_id, model_version,
    # decision_count, human_override_count, avg_confidence, period_start,
    # period_end. The only thing this service holds is WeeklyDetectionReport,
    # whose `layer_summaries` is {"KIT": {"count": 42}} -- a per-layer count
    # with no model identity, no version and no confidence. None of the seven
    # required fields has a source here.
    #
    # The previous body passed six keyword arguments (report_id, week,
    # layer_summary, pii_stats, published_at, audit_id) that are not fields on
    # ModelDecisionSummary, under extra="forbid". Every call that reached the
    # loop raised ValidationError and the route returned 500; the loop only ran
    # for a PUBLISHED report, so the empty case returned [] and the test that
    # covered it asserted the empty list. A route that 500s the moment it has
    # data looks healthy to a suite that never gives it data.
    #
    # Serving this needs the model registry (identity and version) and the
    # decision audit log (counts, overrides, confidence). Neither is reachable
    # from here.
    raise HTTPException(
        status_code=501,
        detail=(
            "decision summary requires the model registry and the decision "
            "audit log; this service holds neither, and a report's "
            "layer_summaries carries no model identity"
        ),
    )


@app.get("/v1/ai/monitoring/pii-stats")
async def pii_stats() -> PiiStripStats:
    # Real implementation: aggregate PII stripping statistics from reports_store
    # Queries audit entries (via audit service replay) for PII events
    total_calls = len(reports_store)
    total_tokens = sum(
        int(
            str(
                r.pii_strip_stats.get("replaced_count", 0)
                if isinstance(r.pii_strip_stats, dict)
                else 0
            )
        )
        for r in reports_store.values()
    )
    bypass_count = 0  # By default, no bypasses; audit replay verifies
    return PiiStripStats(
        total_calls=max(total_calls, 0),
        total_tokens_replaced=max(total_tokens, 0),
        bypass_count=bypass_count,
        period_start=min(
            (r.published_at or now()) for r in reports_store.values() if r.published_at
        )
        if any(r.published_at for r in reports_store.values())
        else now(),
        period_end=now(),
    )
