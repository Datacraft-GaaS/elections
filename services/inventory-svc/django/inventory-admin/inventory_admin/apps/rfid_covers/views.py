"""
HTMX views for RFID cover tag admin surfaces.

Three views:
  box_chain_view    -- full state-transition timeline for a ballot box
  tamper_triage_view -- pending tamper alerts with inline resolve
  reader_health_view -- per-reader last-read, tag-count, battery status
"""

from __future__ import annotations

import hashlib
from typing import Any

from django.contrib.auth.decorators import login_required, permission_required
from django.http import HttpRequest, HttpResponse
from django.shortcuts import get_object_or_404, render
from django.utils import timezone
from django.views.decorators.http import require_GET, require_POST

from .models import CoverStateTransition, CoverTamperAlert, RfidCoverRead, RfidCoverTag


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _verify_chain(transitions: list[CoverStateTransition]) -> list[dict[str, Any]]:
    """Return per-transition verification status by replaying prev_hash chain."""
    result = []
    prev = None
    for t in transitions:
        if prev is None:
            valid = True
        else:
            expected = hashlib.sha256(prev.this_hash.encode()).hexdigest()
            valid = t.prev_hash == expected
        result.append({"transition": t, "chain_valid": valid})
        prev = t
    return result


# ---------------------------------------------------------------------------
# box_chain_view
# ---------------------------------------------------------------------------


@login_required
@permission_required("rfid_covers.view_coverstatetransition", raise_exception=True)
@require_GET
def box_chain_view(request: HttpRequest) -> HttpResponse:
    """Full state-transition timeline for a ballot box.

    Query param: ballot_box_id (box body serial or tag EPC).
    """
    ballot_box_id = request.GET.get("ballot_box_id", "").strip()
    transitions: list[dict[str, Any]] = []
    tag: RfidCoverTag | None = None

    if ballot_box_id:
        try:
            tag = RfidCoverTag.objects.get(paired_with_box=ballot_box_id)
        except RfidCoverTag.DoesNotExist:
            try:
                tag = RfidCoverTag.objects.get(epc=ballot_box_id)
            except RfidCoverTag.DoesNotExist:
                tag = None

        if tag:
            raw = list(
                CoverStateTransition.objects.filter(tag=tag).order_by("transitioned_at")
            )
            transitions = _verify_chain(raw)

    ctx = {
        "ballot_box_id": ballot_box_id,
        "tag": tag,
        "transitions": transitions,
        "now": timezone.now(),
    }
    return render(request, "rfid_covers/box_chain.html", ctx)


# ---------------------------------------------------------------------------
# tamper_triage_view
# ---------------------------------------------------------------------------


@login_required
@permission_required("rfid_covers.view_covertamperalert", raise_exception=True)
def tamper_triage_view(request: HttpRequest) -> HttpResponse:
    """Pending tamper alerts with inline resolve."""
    if request.method == "POST":
        return _resolve_alert(request)

    severity = request.GET.get("severity", "")
    alerts_qs = CoverTamperAlert.objects.filter(
        resolved_at__isnull=True
    ).select_related("tag")
    if severity:
        alerts_qs = alerts_qs.filter(severity=severity)

    ctx = {
        "alerts": alerts_qs.order_by("-detected_at"),
        "severity_choices": CoverTamperAlert.Severity.choices,
        "selected_severity": severity,
    }
    return render(request, "rfid_covers/tamper_triage.html", ctx)


@login_required
@permission_required("rfid_covers.change_covertamperalert", raise_exception=True)
@require_POST
def _resolve_alert(request: HttpRequest) -> HttpResponse:
    alert_id = request.POST.get("alert_id", "")
    note = (
        request.POST.get("resolution_note", "").strip() or f"Resolved by {request.user}"
    )
    alert = get_object_or_404(
        CoverTamperAlert, alert_id=alert_id, resolved_at__isnull=True
    )
    alert.resolved_at = timezone.now()
    alert.resolution_note = note
    alert.save(update_fields=["resolved_at", "resolution_note"])
    remaining = CoverTamperAlert.objects.filter(resolved_at__isnull=True).count()
    ctx = {"resolved_alert_id": alert_id, "remaining": remaining}
    return render(request, "rfid_covers/tamper_triage.html#resolved_fragment", ctx)


# ---------------------------------------------------------------------------
# reader_health_view
# ---------------------------------------------------------------------------


@login_required
@permission_required("rfid_covers.view_rfidcoverread", raise_exception=True)
@require_GET
def reader_health_view(request: HttpRequest) -> HttpResponse:
    """Per-reader last-read timestamp, tag count, and battery status."""
    from django.db.models import Count, Max

    reader_stats = (
        RfidCoverRead.objects.values("reader_serial", "reader_type")
        .annotate(
            tag_count=Count("read_id"),
            last_read=Max("read_at"),
        )
        .order_by("-last_read")
    )

    ctx = {
        "reader_stats": list(reader_stats),
        "now": timezone.now(),
    }
    return render(request, "rfid_covers/reader_health.html", ctx)
