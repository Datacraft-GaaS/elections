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

"""Views for declaration-admin (WP-3.3, M51, M63).

Role gates:
  - Chair:  sign-off on declarations and certificates.
  - CRO:    compile form34C and resolve reconciliation holds.
  - RO:     dispute review and declaration workflow.
"""

from __future__ import annotations

import functools
from typing import Any

from django.contrib.auth.decorators import login_required
from django.http import HttpRequest, HttpResponse, HttpResponseForbidden
from django.shortcuts import get_object_or_404, redirect, render

from .models import (
    CertificateOfElection,
    Declaration,
    PetitionWindow,
    ReconciliationHold,
)
from .settings import ROLE_CHAIR, ROLE_CRO, ROLE_RO


# ---- Role gate decorator ----------------------------------------------------


def role_required(*roles: str):
    """Decorator: reject with 403 if user does not carry one of the required roles.

    Roles are stored as groups on the Django user.  For now the check uses
    group names; a SpiceDB integration replaces this in Wave 4.
    """

    def decorator(view_func):
        @functools.wraps(view_func)
        def wrapper(request: HttpRequest, *args: Any, **kwargs: Any) -> HttpResponse:
            user_roles = set(request.user.groups.values_list("name", flat=True))
            if not user_roles.intersection(roles):
                return HttpResponseForbidden("Insufficient role for this action.")
            return view_func(request, *args, **kwargs)

        return wrapper

    return decorator


# ---- Declarations queue -----------------------------------------------------


@login_required
@role_required(ROLE_CHAIR, ROLE_CRO, ROLE_RO)
def declarations_queue(request: HttpRequest) -> HttpResponse:
    """Declarations pending review or awaiting Chair sign-off (FR-M51-060)."""
    status_filter = request.GET.get("status", "pending")
    county = request.GET.get("county", "")
    seat_type = request.GET.get("seat_type", "")

    qs = Declaration.objects.all()
    if status_filter:
        qs = qs.filter(status=status_filter)
    if county:
        qs = qs.filter(contest_id__startswith=county)
    if seat_type:
        qs = qs.filter(contest_id__contains=seat_type)

    return render(
        request,
        "declaration_admin/declarations_queue.html",
        {
            "declarations": qs,
            "status_filter": status_filter,
            "county": county,
            "seat_type": seat_type,
        },
    )


@login_required
@role_required(ROLE_CHAIR, ROLE_CRO, ROLE_RO)
def declaration_detail(request: HttpRequest, declaration_id: str) -> HttpResponse:
    """Single declaration detail with audit trail."""
    decl = get_object_or_404(Declaration, id=declaration_id)
    amendments = decl.amendments.order_by("amended_at")
    certificates = decl.certificates.order_by("-created_at")
    petition_window = getattr(decl, "petition_window", None)

    return render(
        request,
        "declaration_admin/declaration_detail.html",
        {
            "declaration": decl,
            "amendments": amendments,
            "certificates": certificates,
            "petition_window": petition_window,
        },
    )


# ---- Certificate lookup + issuance ------------------------------------------


@login_required
@role_required(ROLE_CHAIR, ROLE_CRO)
def certificate_panel(request: HttpRequest) -> HttpResponse:
    """Certificate lookup and issuance panel.

    Chair can trigger issuance; CRO can view.
    """
    certs = CertificateOfElection.objects.select_related("declaration").order_by(
        "-created_at"
    )
    return render(
        request, "declaration_admin/certificate_panel.html", {"certificates": certs}
    )


@login_required
@role_required(ROLE_CHAIR)
def issue_certificate(request: HttpRequest, declaration_id: str) -> HttpResponse:
    """Trigger HSM-signed certificate issuance for a DECLARED declaration.

    In the stub, this calls out to the Rust API's /declare endpoint.
    Full integration in Wave 4 with HSM ceremony flow.
    """
    decl = get_object_or_404(Declaration, id=declaration_id, status="declared")
    if request.method == "POST":
        # POST to rts-api to trigger certificate issuance.
        # Stub: redirect back with a success message.
        return redirect("declaration_admin:certificate_panel")

    return render(
        request, "declaration_admin/issue_certificate.html", {"declaration": decl}
    )


# ---- Petition window monitor ------------------------------------------------


@login_required
@role_required(ROLE_CHAIR, ROLE_CRO, ROLE_RO)
def petition_window_monitor(request: HttpRequest) -> HttpResponse:
    """Petition window monitor: T+0 through T+28d per s.75 (EC-TIM-08)."""
    windows = PetitionWindow.objects.select_related("declaration").order_by(
        "window_close_at"
    )
    return render(
        request,
        "declaration_admin/petition_window_monitor.html",
        {"windows": windows},
    )


# ---- Correction workflow ----------------------------------------------------


@login_required
@role_required(ROLE_CHAIR, ROLE_RO)
def correction_workflow(request: HttpRequest, declaration_id: str) -> HttpResponse:
    """Dual sign-off correction workflow (EC-LEG-11, FR-M51-039, FR-M51-040).

    Requires Chair + RO FIDO2 assertions; evidence upload; re-declaration.
    """
    decl = get_object_or_404(Declaration, id=declaration_id)

    if request.method == "POST":
        reason = request.POST.get("reason", "").strip()
        approved_by = request.POST.get("approved_by", "").strip()
        if not reason or not approved_by:
            return render(
                request,
                "declaration_admin/correction_workflow.html",
                {"declaration": decl, "error": "Reason and dual approval required."},
            )
        # Stub: record amendment and redirect.
        return redirect(
            "declaration_admin:declaration_detail", declaration_id=declaration_id
        )

    amendments = decl.amendments.order_by("amended_at")
    return render(
        request,
        "declaration_admin/correction_workflow.html",
        {"declaration": decl, "amendments": amendments},
    )


# ---- Ballot-serial reconciliation holds queue -------------------------------


@login_required
@role_required(ROLE_CHAIR, ROLE_CRO)
def reconciliation_holds_queue(request: HttpRequest) -> HttpResponse:
    """Ballot-serial reconciliation holds queue (M63, FR-M63-048)."""
    holds = ReconciliationHold.objects.filter(result_hold_active=True).order_by(
        "-raised_at"
    )
    return render(
        request,
        "declaration_admin/reconciliation_holds.html",
        {"holds": holds},
    )


# ---- Discrepancy adjudication -----------------------------------------------


@login_required
@role_required(ROLE_CHAIR, ROLE_CRO)
def discrepancy_adjudication(request: HttpRequest, hold_id: str) -> HttpResponse:
    """Adjudicate a ballot-serial discrepancy hold.

    Dual-officer approval required (FR-M63-026): PO and CRO must both sign.
    """
    hold = get_object_or_404(ReconciliationHold, id=hold_id)

    if request.method == "POST":
        resolved_by_po = request.POST.get("resolved_by_po", "").strip()
        resolved_by_cro = request.POST.get("resolved_by_cro", "").strip()
        reason = request.POST.get("reason", "").strip()

        if not resolved_by_po or not resolved_by_cro or not reason:
            return render(
                request,
                "declaration_admin/discrepancy_adjudication.html",
                {
                    "hold": hold,
                    "error": "Both PO and CRO approval with reason required.",
                },
            )

        from django.utils import timezone

        hold.resolved_at = timezone.now()
        hold.resolved_by_po = resolved_by_po
        hold.resolved_by_cro = resolved_by_cro
        hold.resolve_reason = reason
        hold.result_hold_active = False
        hold.save()
        return redirect("declaration_admin:reconciliation_holds_queue")

    return render(
        request,
        "declaration_admin/discrepancy_adjudication.html",
        {"hold": hold},
    )
