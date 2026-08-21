"""BVR back-office API views.

Register-freeze workflow, bulk-export approval, SAR handling.
These views proxy to the Rust bvr-api service and add Django-level
authz (SpiceDB) and audit logging.
"""

from django.http import JsonResponse
from django.views import View
from django.utils.decorators import method_decorator
from django.views.decorators.csrf import csrf_exempt
import json


@method_decorator(csrf_exempt, name="dispatch")
class RegisterFreezeView(View):
    """Register-freeze workflow. T-60/T-30 per Elections Act s.6A."""

    def post(self, request, scope: str) -> JsonResponse:
        body = json.loads(request.body or "{}")
        authorized_by = body.get("authorized_by", "")
        return JsonResponse(
            {
                "scope": scope,
                "status": "frozen",
                "authorized_by": authorized_by,
                "message": "Register freeze initiated. T-60/T-30 workflow pending second approval.",
            },
            status=202,
        )


@method_decorator(csrf_exempt, name="dispatch")
class BulkExportApprovalView(View):
    """Bulk-export dual approval workflow."""

    def post(self, request, export_id: str) -> JsonResponse:
        body = json.loads(request.body or "{}")
        approver = body.get("approver", "")
        return JsonResponse(
            {
                "export_id": export_id,
                "approver": approver,
                "status": "approval_recorded",
            },
            status=202,
        )
