from rest_framework.routers import DefaultRouter

from .views import (
    CourseModuleViewSet,
    TrainingProgressViewSet,
    WorkerCertificateViewSet,
    WorkerEnrollmentViewSet,
)

router = DefaultRouter()
router.register(r"modules", CourseModuleViewSet, basename="coursemodule")
router.register(r"enrollments", WorkerEnrollmentViewSet, basename="workerenrollment")
router.register(r"certificates", WorkerCertificateViewSet, basename="workercertificate")
router.register(r"progress", TrainingProgressViewSet, basename="trainingprogress")

urlpatterns = router.urls

router.register(r"lessons", LessonViewSet, basename="lesson")  # needs import
router.register(r"attempts", LessonAttemptViewSet, basename="lessonattempt")
router.register(r"cert-audit", CertAuditLogViewSet, basename="certauditlog")
