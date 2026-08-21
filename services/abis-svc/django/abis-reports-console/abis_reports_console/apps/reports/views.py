"""Views for the ABIS reports console.

HTMX partials, new-run form, delivery dashboard, subscription wizard.
"""

import hashlib
import io
import json
import uuid
from typing import Any

from django.contrib.auth.decorators import login_required
from django.http import (
    HttpRequest,
    HttpResponse,
    HttpResponseBase,
    JsonResponse,
    StreamingHttpResponse,
)
from django.shortcuts import get_object_or_404, redirect, render
from django.utils import timezone
from django.views.decorators.http import require_http_methods

from .models import (
    ReportAccessLog,
    ReportArtifact,
    ReportDefinition,
    ReportDelivery,
    ReportRun,
    ReportSubscriber,
    ReportSubscription,
)


# ---------------------------------------------------------------------------
# Run detail (HTMX partial)
# ---------------------------------------------------------------------------


@login_required
@require_http_methods(["GET"])
def run_detail(request: HttpRequest, run_id: str) -> HttpResponse:
    """HTMX partial: run side-panel with artifact preview."""
    run = get_object_or_404(ReportRun, pk=run_id)
    artifacts = ReportArtifact.objects.filter(run=run)
    preview: dict[str, Any] = {}

    for artifact in artifacts:
        fmt = artifact.format
        if fmt in ("csv", "json", "xlsx") and artifact.row_count:
            preview[fmt] = {"type": "tabular", "rows": min(20, artifact.row_count or 0)}
        elif fmt == "pdf":
            preview[fmt] = {"type": "pdf", "pages": artifact.page_count}
        elif fmt == "html":
            preview[fmt] = {"type": "html"}
        elif fmt == "png":
            preview[fmt] = {"type": "chart"}

    return render(
        request,
        "reports/partials/run_detail.html",
        {"run": run, "artifacts": artifacts, "preview": preview},
    )


# ---------------------------------------------------------------------------
# New run form
# ---------------------------------------------------------------------------


@login_required
@require_http_methods(["GET", "POST"])
def new_run(request: HttpRequest) -> HttpResponse:
    """Form to queue a new report run."""
    active_defs = ReportDefinition.objects.filter(retired_at__isnull=True).order_by(
        "slug"
    )

    formats_list = ["pdf", "csv", "json", "html", "xlsx", "png"]
    if request.method == "GET":
        return render(
            request,
            "reports/new_run.html",
            {"definitions": active_defs, "formats_list": formats_list},
        )

    slug = request.POST.get("slug", "")
    window_from_str = request.POST.get("window_from", "")
    window_to_str = request.POST.get("window_to", "")
    formats = request.POST.getlist("formats")

    errors: list[str] = []
    if not slug:
        errors.append("Slug is required.")
    if not window_from_str:
        errors.append("Window start is required.")
    if not window_to_str:
        errors.append("Window end is required.")
    if not formats:
        errors.append("At least one format must be selected.")

    if errors:
        return render(
            request,
            "reports/new_run.html",
            {"definitions": active_defs, "errors": errors, "post": request.POST},
            status=400,
        )

    try:
        definition = ReportDefinition.objects.get(slug=slug, retired_at__isnull=True)
    except ReportDefinition.DoesNotExist:
        return render(
            request,
            "reports/new_run.html",
            {
                "definitions": active_defs,
                "errors": [f"Unknown slug: {slug}"],
                "post": request.POST,
            },
            status=400,
        )

    from django.utils.dateparse import parse_datetime

    window_from = parse_datetime(
        window_from_str + ":00" if len(window_from_str) == 16 else window_from_str
    )
    window_to = parse_datetime(
        window_to_str + ":00" if len(window_to_str) == 16 else window_to_str
    )

    if window_from is None or window_to is None:
        return render(
            request,
            "reports/new_run.html",
            {
                "definitions": active_defs,
                "errors": ["Invalid date/time format."],
                "post": request.POST,
            },
            status=400,
        )

    run = ReportRun(
        run_id=uuid.uuid4(),
        definition=definition,
        slug=slug,
        window_from=window_from,
        window_to=window_to,
        status="queued",
        requested_by=str(request.user),
        requested_at=timezone.now(),
    )
    run.save()

    return redirect("reports:run_detail", run_id=str(run.run_id))


# ---------------------------------------------------------------------------
# Delivery dashboard
# ---------------------------------------------------------------------------


@login_required
@require_http_methods(["GET"])
def delivery_dashboard(request: HttpRequest) -> HttpResponse:
    """Table of pending and failed deliveries with retry controls."""
    deliveries = (
        ReportDelivery.objects.filter(status__in=["pending", "failed"])
        .select_related("run", "subscription__subscriber", "artifact")
        .order_by("next_attempt_at")
    )

    return render(request, "reports/deliveries.html", {"deliveries": deliveries})


@login_required
@require_http_methods(["POST"])
def retry_delivery(request: HttpRequest, delivery_id: str) -> HttpResponse:
    """Retry a single failed delivery."""
    delivery = get_object_or_404(
        ReportDelivery, pk=delivery_id, status__in=["failed", "abandoned"]
    )
    delivery.status = "pending"
    delivery.next_attempt_at = timezone.now()
    delivery.save()
    if getattr(request, "htmx", None):
        return HttpResponse(
            f'<span class="badge badge-pending">pending</span>', status=200
        )
    return redirect("reports:delivery_dashboard")


# ---------------------------------------------------------------------------
# Subscription wizard
# ---------------------------------------------------------------------------


@login_required
@require_http_methods(["GET", "POST"])
def subscription_wizard(request: HttpRequest) -> HttpResponse:
    """Two-step subscription wizard: pick subscriber, then pick definitions."""
    step = request.GET.get("step", "1")
    subscribers = ReportSubscriber.objects.filter(suspended_at__isnull=True).order_by(
        "name"
    )
    active_defs = ReportDefinition.objects.filter(retired_at__isnull=True).order_by(
        "slug"
    )

    if request.method == "GET":
        return render(
            request,
            "reports/subscription_wizard.html",
            {"step": step, "subscribers": subscribers, "definitions": active_defs},
        )

    subscriber_id = request.POST.get("subscriber_id", "")
    definition_id = request.POST.get("definition_id", "")
    fmt = request.POST.get("format", "pdf")
    filter_json_raw = request.POST.get("filter_json", "{}")

    errors: list[str] = []
    try:
        filter_json = json.loads(filter_json_raw)
    except json.JSONDecodeError:
        errors.append("Filter JSON is invalid.")
        filter_json = {}

    if not subscriber_id:
        errors.append("Subscriber is required.")
    if not definition_id:
        errors.append("Report definition is required.")

    if errors:
        return render(
            request,
            "reports/subscription_wizard.html",
            {
                "step": "2",
                "subscribers": subscribers,
                "definitions": active_defs,
                "errors": errors,
            },
            status=400,
        )

    subscriber = get_object_or_404(ReportSubscriber, pk=subscriber_id)
    definition = get_object_or_404(ReportDefinition, pk=definition_id)

    sub = ReportSubscription(
        subscription_id=uuid.uuid4(),
        subscriber=subscriber,
        definition=definition,
        format=fmt,
        filter=filter_json,
        created_at=timezone.now(),
    )
    sub.save()

    return render(
        request,
        "reports/subscription_wizard.html",
        {"step": "done", "subscription": sub},
    )


# ---------------------------------------------------------------------------
# Artifact download (records access log)
# ---------------------------------------------------------------------------


@login_required
@require_http_methods(["GET"])
def artifact_download(request: HttpRequest, artifact_id: str) -> HttpResponseBase:
    """Stream an artifact from MinIO and record an access log entry."""
    from django.core.files.storage import default_storage

    artifact = get_object_or_404(ReportArtifact, pk=artifact_id)

    ua_raw = request.META.get("HTTP_USER_AGENT", "")
    ua_hash = hashlib.sha256(ua_raw.encode()).digest()
    ip = request.META.get("REMOTE_ADDR", "")
    ip_prefix = ".".join(ip.split(".")[:3]) + ".0/24" if "." in ip else ip[:12]

    ReportAccessLog.objects.create(
        access_id=uuid.uuid4(),
        artifact=artifact,
        subject_id=str(request.user),
        ip_prefix=ip_prefix,
        user_agent_hash=ua_hash,
        accessed_at=timezone.now(),
        byte_range=request.headers.get("Range"),
    )

    content_types = {
        "pdf": "application/pdf",
        "csv": "text/csv",
        "json": "application/json",
        "xlsx": "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "html": "text/html",
        "png": "image/png",
        "svg": "image/svg+xml",
        "parquet": "application/octet-stream",
    }
    content_type = content_types.get(artifact.format, "application/octet-stream")
    filename = artifact.blob_ref.split("/")[-1]

    from django.http import StreamingHttpResponse

    def _iter():
        with default_storage.open(artifact.blob_ref, "rb") as f:
            while chunk := f.read(65536):
                yield chunk

    resp = StreamingHttpResponse(_iter(), content_type=content_type)
    resp["Content-Disposition"] = f'attachment; filename="{filename}"'
    return resp
