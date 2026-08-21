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
