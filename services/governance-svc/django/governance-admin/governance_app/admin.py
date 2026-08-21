from django.contrib import admin
from .models import (
    AuditRunModel,
    SampledBallotModel,
    AuditReportModel,
    PetitionModel,
    EvidenceBundleModel,
    TransmissionRecordModel,
    ComplianceVerdictModel,
    StatutoryReportModel,
    OagEngagementModel,
    OagResponseModel,
)

_COMMON_LIST = ["pk", "status", "created_at"]
_COMMON_READONLY = ["id", "status", "created_at"]
_COMMON_SEARCH = ["status"]


def _make_admin(model_cls):
    class _Admin(admin.ModelAdmin):
        list_display = _COMMON_LIST
        readonly_fields = _COMMON_READONLY
        search_fields = _COMMON_SEARCH

        def has_add_permission(self, request):
            return False

        def has_delete_permission(self, request, obj=None):
            return False

        def has_change_permission(self, request, obj=None):
            return False

    _Admin.__name__ = f"{model_cls.__name__}Admin"
    return _Admin


for _model in [
    AuditRunModel,
    SampledBallotModel,
    AuditReportModel,
    PetitionModel,
    EvidenceBundleModel,
    TransmissionRecordModel,
    ComplianceVerdictModel,
    StatutoryReportModel,
    OagEngagementModel,
    OagResponseModel,
]:
    admin.site.register(_model, _make_admin(_model))
