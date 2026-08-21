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
