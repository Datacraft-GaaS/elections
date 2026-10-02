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

"""
HTMX views for RFID cover tag admin surfaces.

Three views:
  box_chain_view    -- full state-transition timeline for a ballot box
  tamper_triage_view -- pending tamper alerts with inline resolve
  reader_health_view -- per-reader last-read, tag-count, battery status
"""

from __future__ import annotations

import json
from typing import Any

import blake3
from django.contrib.auth.decorators import login_required, permission_required
from django.http import HttpRequest, HttpResponse
from django.shortcuts import get_object_or_404, render
from django.utils import timezone
from django.views.decorators.http import require_GET, require_POST

from .models import CoverStateTransition, CoverTamperAlert, RfidCoverRead, RfidCoverTag


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _canonical_transition_bytes(t: CoverStateTransition) -> bytes:
    """Serialise a transition exactly as the Rust writer does.

    Mirrors `rfid-cover-custody/src/hash_chain.rs::compute_this_hash`:
    serde_json object with keys in order transition_id, tag_id, from_state,
    to_state, officer_id, officer_svid, transitioned_at; compact separators;
    null for absent optional values; RFC 3339 timestamp.
    """
    payload = {
        "transition_id": str(t.transition_id),
        "tag_id": str(t.tag_id),
        "from_state": t.from_state,
        "to_state": t.to_state,
        "officer_id": str(t.officer_id) if t.officer_id else None,
        "officer_svid": t.officer_svid if t.officer_svid else None,
        "transitioned_at": t.transitioned_at.isoformat(),
    }
    return json.dumps(payload, separators=(",", ":")).encode()


def _verify_chain(transitions: list[CoverStateTransition]) -> list[dict[str, Any]]:
    """Return per-transition verification status by replaying the BLAKE3 chain.

    Matches the Rust writer: this_hash = BLAKE3(prev_hash_bytes || canonical_json).
    """
    result = []
    prev = b""
    for t in transitions:
        expected = blake3.blake3(prev + _canonical_transition_bytes(t)).digest()
        valid = prev == (bytes(t.prev_hash) if t.prev_hash else b"") and bytes(
            t.this_hash
        ) == expected
        result.append({"transition": t, "chain_valid": valid})
        prev = bytes(t.this_hash)
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
    from .dual_control import DualApprovalError, countersigned_note, validate_dual_approval

    alert_id = request.POST.get("alert_id", "")

    # Wave-2 HM1: tamper-alert resolution is dual-control. The second officer
    # is physically co-located (works offline); both ids are recorded.
    try:
        second = validate_dual_approval(str(request.user), request.POST.get("second_officer"))
    except DualApprovalError as e:
        return HttpResponse(str(e), status=422)

    note = countersigned_note(
        str(request.user),
        second,
        request.POST.get("resolution_note", "").strip()
        or f"Resolved by {request.user}",
    )
    alert = get_object_or_404(
        CoverTamperAlert, alert_id=alert_id, resolved_at__isnull=True
    )
    alert.resolved_at = timezone.now()
    alert.resolution_note = note
    alert.save(update_fields=["resolved_at", "resolution_note"])
    remaining = CoverTamperAlert.objects.filter(resolved_at__isnull=True).count()
    ctx = {
        "resolved_alert_id": alert_id,
        "remaining": remaining,
        "countersigned_by": second,
    }
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
