from rest_framework.routers import DefaultRouter

from .views import (
    ContentItemViewSet,
    ContentVersionViewSet,
    DistributionEventViewSet,
    FactCheckReviewViewSet,
)

router = DefaultRouter()
router.register(r"content-items", ContentItemViewSet, basename="contentitem")
router.register(r"content-versions", ContentVersionViewSet, basename="contentversion")
router.register(
    r"fact-check-reviews", FactCheckReviewViewSet, basename="factcheckreview"
)
router.register(
    r"distribution-events", DistributionEventViewSet, basename="distributionevent"
)

urlpatterns = router.urls
