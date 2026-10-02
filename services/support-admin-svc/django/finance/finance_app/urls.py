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
