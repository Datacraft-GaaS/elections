from django.urls import include, path
from rest_framework.routers import DefaultRouter

from . import views

router = DefaultRouter()
router.register("budgets", views.BudgetViewSet, basename="budget")
router.register("budget-lines", views.BudgetLineViewSet, basename="budgetline")
router.register("procurements", views.ProcurementRecordViewSet, basename="procurement")
router.register("vouchers", views.PaymentVoucherViewSet, basename="voucher")
router.register("cob-reports", views.CoBReportViewSet, basename="cobreport")

urlpatterns = [
    path("v1/finance/", include(router.urls)),
]
