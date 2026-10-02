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
from accreditation_app.views import (
    MediaOrganisationViewSet,
    MediaAccreditationViewSet,
    ObserverOrganisationViewSet,
    IndividualObserverViewSet,
    verify_badge_view,
    crl_view,
)

router = DefaultRouter()
router.register(
    r"media-organisations", MediaOrganisationViewSet, basename="media-organisation"
)
router.register(
    r"media-accreditations", MediaAccreditationViewSet, basename="media-accreditation"
)
router.register(
    r"observer-organisations",
    ObserverOrganisationViewSet,
    basename="observer-organisation",
)
router.register(r"observers", IndividualObserverViewSet, basename="observer")

urlpatterns = [
    path("", include(router.urls)),
    path("verify-badge/<str:serial>/", verify_badge_view, name="verify-badge"),
    path("crl/", crl_view, name="crl"),
]
