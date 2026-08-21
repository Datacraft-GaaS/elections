"""HTMX-driven views for the adjudicator console."""

import hashlib

from django.contrib.auth.decorators import login_required
from django.http import HttpRequest, HttpResponse, HttpResponseBadRequest
from django.shortcuts import get_object_or_404, render
from django.views.decorators.http import require_GET, require_POST

from .models import AdjudicationCase, ReviewerAction


@login_required
@require_GET
def case_list(request: HttpRequest) -> HttpResponse:
    """List open adjudication cases ordered by priority and SLA deadline."""
    cases = AdjudicationCase.objects.exclude(
        state__in=["Decided", "Withdrawn"]
    ).order_by("-priority", "sla_deadline_at")
    return render(request, "adjudication/case_list.html", {"cases": cases})


@login_required
@require_GET
def case_detail(request: HttpRequest, case_id: str) -> HttpResponse:
    """HTMX-driven case detail: side-by-side probe vs gallery, similarity scores."""
    case = get_object_or_404(AdjudicationCase, pk=case_id)
    actions = ReviewerAction.objects.filter(case=case).order_by("taken_at")

    from django.conf import settings

    evidence_base = getattr(settings, "MINIO_EVIDENCE_BASE_URL", "")
    # Signed URL construction would use MinIO presigned URLs in production.
    # Here we build a placeholder URL so the template can render.
    evidence_url = f"{evidence_base}/{case.evidence_bundle_ref}"

    context = {
        "case": case,
        "actions": actions,
        "evidence_url": evidence_url,
        "is_decided": case.is_decided(),
        "sla_hours": case.sla_hours_remaining(),
        "dual_signoff_required": case.tier == "Tier3Panel",
    }

    if request.htmx:  # type: ignore[attr-defined]
        return render(request, "adjudication/partials/case_detail_panel.html", context)
    return render(request, "adjudication/case_detail.html", context)


@login_required
@require_POST
def submit_decision(request: HttpRequest, case_id: str) -> HttpResponse:
    """Submit an adjudication decision with optional dual sign-off for Tier3."""
    case = get_object_or_404(AdjudicationCase, pk=case_id)

    if case.is_decided():
        return HttpResponseBadRequest("Case is already decided.")

    decision_type = request.POST.get("decision_type")
    if not decision_type:
        return HttpResponseBadRequest("decision_type is required.")

    if case.tier == "Tier3Panel":
        # Dual sign-off: require a second_reviewer_id in the form.
        second_reviewer = request.POST.get("second_reviewer_id")
        if not second_reviewer:
            return HttpResponseBadRequest("Tier 3 decisions require dual sign-off.")

    # In production: write a ReviewerAction via the Rust API (not directly via ORM
    # since eel_reviewer_actions has an append-only trigger). Here we render feedback.
    context = {
        "case": case,
        "decision_type": decision_type,
        "submitted": True,
    }
    if request.htmx:  # type: ignore[attr-defined]
        return render(request, "adjudication/partials/decision_confirmed.html", context)
    return render(request, "adjudication/decision_confirmed.html", context)


@login_required
@require_POST
def upload_evidence(request: HttpRequest, case_id: str) -> HttpResponse:
    """Upload a piece of evidence and compute its BLAKE3-equivalent hash.

    The hash is written to eel_reviewer_actions via the Rust ABIS API in production.
    Here we compute and return it for the HTMX swap.
    """
    case = get_object_or_404(AdjudicationCase, pk=case_id)
    evidence_file = request.FILES.get("evidence")
    if not evidence_file:
        return HttpResponseBadRequest("No evidence file provided.")

    content = evidence_file.read()
    # SHA-256 used here; production path writes BLAKE3 via Rust service.
    evidence_hash = hashlib.sha256(content).hexdigest()

    context = {
        "case": case,
        "evidence_hash": evidence_hash,
        "filename": evidence_file.name,
    }
    if request.htmx:  # type: ignore[attr-defined]
        return render(request, "adjudication/partials/evidence_uploaded.html", context)
    return render(request, "adjudication/evidence_uploaded.html", context)
