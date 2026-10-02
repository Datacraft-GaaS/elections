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

"""URL routing for hr_app (WP-5.6)."""

from django.urls import include, path
from rest_framework.routers import DefaultRouter

from .views import (
    JobApplicationViewSet,
    PayrollRunViewSet,
    PayslipLineViewSet,
    StaffContractViewSet,
    StatutoryDeductionViewSet,
)

router = DefaultRouter()
router.register(r"v1/hr/applications", JobApplicationViewSet, basename="application")
router.register(r"v1/hr/contracts", StaffContractViewSet, basename="contract")
router.register(r"v1/payroll/runs", PayrollRunViewSet, basename="payroll-run")
router.register(r"v1/payroll/payslips", PayslipLineViewSet, basename="payslip")
router.register(
    r"v1/payroll/deductions", StatutoryDeductionViewSet, basename="deduction"
)

urlpatterns = [
    path("", include(router.urls)),
]
