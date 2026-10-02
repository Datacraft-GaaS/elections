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

"""Views for ad-hoc report authoring."""

import logging

import httpx
import sqlparse
from django.conf import settings
from django.http import HttpRequest, HttpResponse
from django.shortcuts import render
from django.views.decorators.http import require_GET, require_http_methods, require_POST

logger = logging.getLogger(__name__)

_ADHOC_API_URL = getattr(
    settings, "EEL_ADHOC_API_URL", "http://localhost:8080/v1/adhoc"
)


def _api_client() -> httpx.Client:
    return httpx.Client(base_url=_ADHOC_API_URL, timeout=30.0)


# ---------------------------------------------------------------------------
# List view
# ---------------------------------------------------------------------------


@require_GET
def report_list(request: HttpRequest) -> HttpResponse:
    """List all active ad-hoc reports visible to the current user."""
    scope = request.GET.get("scope", "my")  # my | shared | team | org
    search = request.GET.get("q", "").strip()

    from .models import AdhocReport

    try:
        qs = AdhocReport.objects.filter(archived_at__isnull=True)
        if scope == "my":
            qs = qs.filter(
                author_id=str(request.user.pk) if request.user.is_authenticated else ""
            )
        elif scope == "shared":
            qs = qs.filter(visibility="shared")
        elif scope == "team":
            qs = qs.filter(visibility="team")
        elif scope == "org":
            qs = qs.filter(visibility="org")
        if search:
            qs = qs.filter(title__icontains=search)
        reports = list(qs)
    except Exception:
        reports = []

    return render(
        request,
        "authoring/report_list.html",
        {
            "reports": reports,
            "scope": scope,
            "search": search,
        },
    )


# ---------------------------------------------------------------------------
# Editor view
# ---------------------------------------------------------------------------


@require_GET
def report_editor(request: HttpRequest, adhoc_id: str) -> HttpResponse:
    """Three-pane editor: metadata, source (SQL/JSON), preview."""
    from .models import AdhocReport

    try:
        report = AdhocReport.objects.get(pk=adhoc_id)
    except Exception:
        report = None

    version = None
    highlighted_source = ""
    if report:
        try:
            version = report.versions.order_by("-version_number").first()
            if version:
                highlighted_source = sqlparse.format(
                    version.source,
                    reindent=True,
                    keyword_case="upper",
                )
        except Exception as exc:
            logger.warning("adhoc source highlight failed for %s: %s", adhoc_id, exc)

    return render(
        request,
        "authoring/editor.html",
        {
            "report": report,
            "adhoc_id": adhoc_id,
            "version": version,
            "highlighted_source": highlighted_source,
        },
    )


# ---------------------------------------------------------------------------
# HTMX: save version
# ---------------------------------------------------------------------------


@require_POST
def htmx_save_version(request: HttpRequest, adhoc_id: str) -> HttpResponse:
    """POST to Rust adhoc API, receive lint_report, render inline list."""
    source = request.POST.get("source", "")
    source_type = request.POST.get("source_type", "sql")
    commit_message = request.POST.get("commit_message", "")
    allowed_tables_raw = request.POST.get("allowed_tables", "")
    allowed_tables = [t.strip() for t in allowed_tables_raw.split(",") if t.strip()]

    payload = {
        "source_type": source_type,
        "source": source,
        "allowed_tables": allowed_tables,
        "commit_message": commit_message or None,
    }

    lint_report: dict = {}
    safety_score: float = 0.0
    error_msg: str | None = None

    try:
        with _api_client() as client:
            resp = client.post(f"/reports/{adhoc_id}/versions", json=payload)
            resp.raise_for_status()
            data = resp.json()
            lint_report = data.get("lint_report", {})
            safety_score = data.get("safety_score", 0.0)
    except Exception as exc:
        logger.warning("adhoc API save failed: %s", exc)
        error_msg = str(exc)
        lint_report = {"errors": [], "warnings": []}

    return render(
        request,
        "authoring/partials/lint_report.html",
        {
            "lint_report": lint_report,
            "safety_score": safety_score,
            "error_msg": error_msg,
        },
    )


# ---------------------------------------------------------------------------
# HTMX: run preview
# ---------------------------------------------------------------------------


@require_POST
def htmx_preview(request: HttpRequest, version_id: str) -> HttpResponse:
    """POST preview request, receive sample rows, render table."""
    format_ = request.POST.get("format", "json")
    sample_limit_raw = request.POST.get("sample_limit", "100")
    try:
        sample_limit = min(int(sample_limit_raw), 1000)
    except ValueError:
        sample_limit = 100

    payload = {
        "format": format_,
        "sample_limit": sample_limit,
    }

    rows: list = []
    headers: list[str] = []
    error_msg: str | None = None
    preview_id: str | None = None

    try:
        with _api_client() as client:
            resp = client.post(f"/versions/{version_id}/preview", json=payload)
            resp.raise_for_status()
            data = resp.json()
            preview_id = data.get("preview_id")
            rows = data.get("rows", [])
            if rows:
                headers = list(rows[0].keys()) if isinstance(rows[0], dict) else []
    except Exception as exc:
        logger.warning("adhoc API preview failed: %s", exc)
        error_msg = str(exc)

    return render(
        request,
        "authoring/partials/preview_table.html",
        {
            "rows": rows,
            "headers": headers,
            "error_msg": error_msg,
            "preview_id": preview_id,
            "format": format_,
        },
    )


# ---------------------------------------------------------------------------
# Publish wizard
# ---------------------------------------------------------------------------


@require_http_methods(["GET", "POST"])
def publish_wizard(request: HttpRequest, version_id: str) -> HttpResponse:
    """Pick schedule, subscribers, formats, then POST to /publish."""
    if request.method == "GET":
        return render(
            request,
            "authoring/publish_wizard.html",
            {
                "version_id": version_id,
            },
        )

    schedule = request.POST.get("schedule", "")
    formats = request.POST.getlist("formats")
    payload: dict = {"formats": formats}
    if schedule:
        payload["schedule"] = schedule

    definition_id: str | None = None
    error_msg: str | None = None

    try:
        with _api_client() as client:
            resp = client.post(f"/versions/{version_id}/publish", json=payload)
            resp.raise_for_status()
            data = resp.json()
            definition_id = str(data.get("definition_id", ""))
    except Exception as exc:
        logger.warning("adhoc API publish failed: %s", exc)
        error_msg = str(exc)

    return render(
        request,
        "authoring/partials/publish_result.html",
        {
            "definition_id": definition_id,
            "error_msg": error_msg,
        },
    )
