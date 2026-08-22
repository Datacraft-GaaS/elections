"""
HTMX-driven views for the telemetry admin console.

Three views:
  hard_questions_view   -- preset query library with parameter forms + Run button
  query_builder_view    -- interactive DSL builder + NL bridge
  evidence_export_view  -- queue export, poll status, download bundle
"""

from __future__ import annotations

import json
import logging

import requests
from django.conf import settings
from django.contrib.auth.decorators import login_required
from django.http import HttpRequest, HttpResponse, JsonResponse
from django.shortcuts import render
from django.views.decorators.http import require_GET, require_POST

_log = logging.getLogger(__name__)

TELEMETRY_API_BASE: str = getattr(
    settings, "EEL_TELEMETRY_API_URL", "http://telemetry-query:8080"
)


def _api_actor(request: HttpRequest) -> str:
    """Extract actor SVID from session or fall back to username."""
    svid = request.session.get("actor_svid")
    if svid:
        return svid
    if request.user.is_authenticated:
        return f"django://{request.user.username}"
    return "anonymous"


def _api_get(path: str, params: dict | None = None) -> dict:
    try:
        resp = requests.get(f"{TELEMETRY_API_BASE}{path}", params=params, timeout=30)
        resp.raise_for_status()
        return resp.json()
    except Exception as exc:
        _log.error("telemetry API GET %s failed: %s", path, exc)
        return {"error": str(exc)}


def _api_post(path: str, body: dict, headers: dict | None = None) -> dict:
    try:
        hdrs = {"Content-Type": "application/json", **(headers or {})}
        resp = requests.post(
            f"{TELEMETRY_API_BASE}{path}", json=body, headers=hdrs, timeout=60
        )
        resp.raise_for_status()
        return resp.json()
    except Exception as exc:
        _log.error("telemetry API POST %s failed: %s", path, exc)
        return {"error": str(exc)}


# ---------------------------------------------------------------------------
# View 1: Hard questions (preset library)
# ---------------------------------------------------------------------------


@login_required
@require_GET
def hard_questions_view(request: HttpRequest) -> HttpResponse:
    """List preset queries with parameter forms."""
    presets_data = _api_get("/v1/telemetry/presets")
    presets = presets_data.get("presets", [])
    return render(
        request,
        "telemetry/hard_questions.html",
        {"presets": presets, "page_title": "Hard Questions - Preset Query Library"},
    )


@login_required
@require_POST
def run_preset_view(request: HttpRequest, name: str) -> HttpResponse:
    """Run a named preset with parameters from the POST body."""
    actor = _api_actor(request)
    try:
        params = json.loads(request.body)
    except json.JSONDecodeError:
        return JsonResponse({"error": "invalid JSON body"}, status=400)

    result = _api_post(
        f"/v1/telemetry/presets/{name}/run",
        {"params": params},
        {"X-Actor-Svid": actor},
    )

    if request.headers.get("HX-Request"):
        return render(
            request,
            "telemetry/partials/query_result.html",
            {"result": result, "preset_name": name},
        )
    return JsonResponse(result)


# ---------------------------------------------------------------------------
# View 2: Interactive query builder
# ---------------------------------------------------------------------------


@login_required
@require_GET
def query_builder_view(request: HttpRequest) -> HttpResponse:
    """Interactive DSL builder with signal-class checkboxes + NL bridge."""
    signal_classes = [
        ("location", "Location"),
        ("time", "Time"),
        ("activity", "Activity"),
        ("proximal_rfid", "Proximal RFID"),
        ("movements", "Movements"),
        ("login_logout", "Login / Logout"),
        ("reads", "Reads"),
        ("writes", "Writes"),
        ("battery", "Battery"),
        ("locks_unlocks", "Locks / Unlocks"),
        ("physical_drops", "Physical Drops"),
        ("network", "Network"),
        ("attestation", "Attestation"),
    ]
    return render(
        request,
        "telemetry/query_builder.html",
        {
            "signal_classes": signal_classes,
            "page_title": "Interactive Query Builder",
        },
    )


@login_required
@require_POST
def run_structured_query_view(request: HttpRequest) -> HttpResponse:
    """Execute a structured DSL query from the builder."""
    actor = _api_actor(request)
    try:
        body = json.loads(request.body)
    except json.JSONDecodeError:
        return JsonResponse({"error": "invalid JSON body"}, status=400)

    result = _api_post("/v1/telemetry/query", body, {"X-Actor-Svid": actor})

    if request.headers.get("HX-Request"):
        return render(
            request,
            "telemetry/partials/query_result.html",
            {"result": result, "preset_name": "custom"},
        )
    return JsonResponse(result)


@login_required
@require_POST
def nl_translate_view(request: HttpRequest) -> HttpResponse:
    """Translate natural-language text to a proposed DSL (confirmation required)."""
    actor = _api_actor(request)
    try:
        body = json.loads(request.body)
    except json.JSONDecodeError:
        return JsonResponse({"error": "invalid JSON body"}, status=400)

    text = body.get("text", "").strip()
    if not text:
        return JsonResponse({"error": "text is required"}, status=400)

    result = _api_post(
        "/v1/telemetry/query/nl", {"text": text}, {"X-Actor-Svid": actor}
    )

    if request.headers.get("HX-Request"):
        return render(
            request,
            "telemetry/partials/nl_proposed.html",
            {"result": result},
        )
    return JsonResponse(result)


# ---------------------------------------------------------------------------
# View 3: Evidence export
# ---------------------------------------------------------------------------


@login_required
@require_GET
def evidence_export_view(request: HttpRequest) -> HttpResponse:
    """List recent queries and allow evidence bundle export."""
    from .models import TelemetryQueryAudit

    recent_queries = TelemetryQueryAudit.objects.order_by("-executed_at")[:50]
    return render(
        request,
        "telemetry/evidence_export.html",
        {
            "recent_queries": recent_queries,
            "page_title": "Evidence Export",
        },
    )


@login_required
@require_POST
def trigger_export_view(request: HttpRequest, query_id: str) -> HttpResponse:
    """Trigger evidence bundle export for a query."""
    actor = _api_actor(request)
    result = _api_post(
        f"/v1/telemetry/queries/{query_id}/export",
        {},
        {"X-Actor-Svid": actor},
    )

    if request.headers.get("HX-Request"):
        return render(
            request,
            "telemetry/partials/export_result.html",
            {"result": result, "query_id": query_id},
        )
    return JsonResponse(result)
