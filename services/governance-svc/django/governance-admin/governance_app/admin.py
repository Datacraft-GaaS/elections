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


def _make_admin(model_cls):
    # Derived from the model, not a fixed triple. The tables do not share a
    # shape: `rla_sampled_ballots` has no `status` and no `created_at`, and
    # `oag_responses` has neither either. A hardcoded `["pk", "status",
    # "created_at"]` fails the admin system check with E108 on those, which is
    # how the shared list was found to be wrong.
    fields = [f.name for f in model_cls._meta.get_fields()]
    columns = ["pk"] + [
        c for c in ("status", "created_at", "recorded_at") if c in fields
    ]

    class _Admin(admin.ModelAdmin):
        list_display = columns
        readonly_fields = [f for f in columns if f != "pk"]
        search_fields = [c for c in ("status", "verdict", "conclusion") if c in fields]

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
