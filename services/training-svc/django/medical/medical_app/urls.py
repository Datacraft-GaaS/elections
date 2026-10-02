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

from django.urls import path, include
from rest_framework.routers import DefaultRouter
from medical_app.views import (
    MedicalIncidentViewSet,
    DispatchRequestViewSet,
    ClusterAlertViewSet,
    MedicalUnitViewSet,
)

router = DefaultRouter()
router.register(r"incidents", MedicalIncidentViewSet, basename="incident")
router.register(r"dispatches", DispatchRequestViewSet, basename="dispatch")
router.register(r"cluster-alerts", ClusterAlertViewSet, basename="cluster-alert")
router.register(r"units", MedicalUnitViewSet, basename="unit")

urlpatterns = [
    path("", include(router.urls)),
]
