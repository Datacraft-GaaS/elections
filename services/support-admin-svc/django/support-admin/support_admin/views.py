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

"""Views for support-admin (WP-5.5, M25, M27).

Role gates:
  - COMMISSION_OPS / CHAIR: special facility management and s.44A queue.
  - IAB_AUDITOR: s.44A sample audit, adjudication audit panel.
  - RO / COMMISSION_OPS: rejection rate dashboard.
"""

from __future__ import annotations

import functools
from typing import Any

from django.contrib.auth.decorators import login_required
from django.http import HttpRequest, HttpResponse, HttpResponseForbidden
from django.shortcuts import get_object_or_404, render

from .models import (
    AdjudicationCatalogueEntry,
    AdjudicationEvent,
    IDPReassignment,
    RejectionRateAlert,
    S44AOverrideRequest,
    SpecialVoterFacility,
)
from .settings import (
    REJECTION_RATE_Z_THRESHOLD,
    ROLE_CHAIR,
    ROLE_COMMISSION_OPS,
    ROLE_IAB_AUDITOR,
    ROLE_RO,
)


def role_required(*roles: str):
    """Decorator: reject with 403 if user lacks required role."""

    def decorator(view_func):
        @functools.wraps(view_func)
        def wrapper(request: HttpRequest, *args: Any, **kwargs: Any) -> HttpResponse:
            user_roles = set(request.user.groups.values_list("name", flat=True))
            if not user_roles.intersection(roles):
                return HttpResponseForbidden("Insufficient role.")
            return view_func(request, *args, **kwargs)

        return wrapper

    return decorator


# ---- M25: Facility Management -----------------------------------------------


@login_required
@role_required(ROLE_COMMISSION_OPS, ROLE_CHAIR)
def facility_list(request: HttpRequest) -> HttpResponse:
    """Special voter facilities dashboard (FR-M25-047)."""
    category = request.GET.get("category", "")
    county = request.GET.get("county", "")
    state = request.GET.get("state", "")

    qs = SpecialVoterFacility.objects.all()
    if category:
        qs = qs.filter(category=category)
    if county:
        qs = qs.filter(county_code=county)
    if state:
        qs = qs.filter(session_state=state)

    return render(
        request,
        "support_admin/facility_list.html",
        {"facilities": qs, "category": category, "county": county, "state": state},
    )


@login_required
@role_required(ROLE_COMMISSION_OPS, ROLE_CHAIR)
def facility_detail(request: HttpRequest, facility_id: str) -> HttpResponse:
    """Facility detail with s.44A override history and IDP reassignments."""
    facility = get_object_or_404(SpecialVoterFacility, pk=facility_id)
    overrides = S44AOverrideRequest.objects.filter(facility=facility)
    reassignments = IDPReassignment.objects.filter(idp_facility=facility)
    return render(
        request,
        "support_admin/facility_detail.html",
        {
            "facility": facility,
            "overrides": overrides,
            "reassignments": reassignments,
        },
    )


# ---- M25: S.44A Override Queue ----------------------------------------------


@login_required
@role_required(ROLE_IAB_AUDITOR, ROLE_COMMISSION_OPS)
def s44a_override_queue(request: HttpRequest) -> HttpResponse:
    """IAB s.44A override review queue (FR-M25-012)."""
    pending = S44AOverrideRequest.objects.filter(approved=False)
    approved = S44AOverrideRequest.objects.filter(approved=True)
    return render(
        request,
        "support_admin/s44a_queue.html",
        {"pending": pending, "approved": approved},
    )


# ---- M27: Adjudication Catalogue Browser ------------------------------------


@login_required
@role_required(ROLE_COMMISSION_OPS, ROLE_IAB_AUDITOR, ROLE_RO)
def adjudication_catalogue(request: HttpRequest) -> HttpResponse:
    """Adjudication reason-code catalogue browser (M27, A1-A8 / V1-V3)."""
    codes = AdjudicationCatalogueEntry.objects.all()
    decision_filter = request.GET.get("decision", "")
    if decision_filter:
        codes = codes.filter(decision=decision_filter)
    return render(
        request,
        "support_admin/adjudication_catalogue.html",
        {"codes": codes, "decision_filter": decision_filter},
    )


@login_required
@role_required(ROLE_IAB_AUDITOR, ROLE_COMMISSION_OPS)
def adjudication_event_list(request: HttpRequest) -> HttpResponse:
    """Adjudication audit panel for IAB (M27)."""
    station = request.GET.get("station", "")
    decision = request.GET.get("decision", "")
    qs = AdjudicationEvent.objects.all()
    if station:
        qs = qs.filter(station_code=station)
    if decision:
        qs = qs.filter(decision=decision)
    return render(
        request,
        "support_admin/adjudication_event_list.html",
        {"events": qs, "station": station, "decision": decision},
    )


# ---- M27: Rejection Rate Dashboard ------------------------------------------


@login_required
@role_required(ROLE_COMMISSION_OPS, ROLE_RO, ROLE_CHAIR)
def rejection_rate_dashboard(request: HttpRequest) -> HttpResponse:
    """Rejection-rate outlier dashboard (M27, ECC-facing).

    Lists stations with |z| >= threshold, newest first.
    """
    qs = RejectionRateAlert.objects.filter(z_score__gte=REJECTION_RATE_Z_THRESHOLD)
    unacknowledged = qs.filter(acknowledged=False)
    acknowledged = qs.filter(acknowledged=True)
    return render(
        request,
        "support_admin/rejection_rate_dashboard.html",
        {
            "unacknowledged": unacknowledged,
            "acknowledged": acknowledged,
            "z_threshold": REJECTION_RATE_Z_THRESHOLD,
        },
    )
