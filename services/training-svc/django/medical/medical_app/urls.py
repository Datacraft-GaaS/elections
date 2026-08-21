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
