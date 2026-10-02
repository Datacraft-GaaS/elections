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

from decimal import Decimal

from django.utils import timezone
from django_filters.rest_framework import DjangoFilterBackend
from rest_framework import status, viewsets
from rest_framework.decorators import action
from rest_framework.filters import OrderingFilter
from rest_framework.response import Response

from .models import (
    CourseModule,
    EnrollmentStatus,
    TrainingProgress,
    WorkerCertificate,
    WorkerEnrollment,
)
from .serializers import (
    AssessmentSubmitSerializer,
    CourseModuleSerializer,
    TrainingProgressSerializer,
    WorkerCertificateSerializer,
    WorkerEnrollmentSerializer,
)


class CourseModuleViewSet(viewsets.ModelViewSet):
    queryset = CourseModule.objects.all()
    serializer_class = CourseModuleSerializer
    filter_backends = [DjangoFilterBackend, OrderingFilter]
    filterset_fields = ["target_audience", "module_type", "language"]
    ordering_fields = ["title", "published_at"]


class WorkerEnrollmentViewSet(viewsets.ModelViewSet):
    queryset = WorkerEnrollment.objects.all()
    serializer_class = WorkerEnrollmentSerializer
    filter_backends = [DjangoFilterBackend]
    filterset_fields = ["worker_id", "module", "status", "certified"]

    @action(detail=True, methods=["post"], url_path="submit-assessment")
    def submit_assessment(self, request, pk=None):
        enrollment = self.get_object()
        serializer = AssessmentSubmitSerializer(data=request.data)
        serializer.is_valid(raise_exception=True)
        score_pct = Decimal(str(serializer.validated_data["score_pct"]))

        enrollment.attempts += 1
        enrollment.status = EnrollmentStatus.IN_PROGRESS

        if score_pct >= enrollment.module.pass_mark_pct:
            enrollment.status = EnrollmentStatus.PASSED
            enrollment.certified = True
            enrollment.completed_at = timezone.now()
            if score_pct > enrollment.best_score_pct:
                enrollment.best_score_pct = score_pct
        else:
            if score_pct > enrollment.best_score_pct:
                enrollment.best_score_pct = score_pct
            if enrollment.attempts >= enrollment.module.max_retakes:
                enrollment.status = EnrollmentStatus.FAILED
            else:
                enrollment.status = EnrollmentStatus.RETAKE_REQUIRED

        enrollment.save()
        return Response(
            WorkerEnrollmentSerializer(enrollment).data, status=status.HTTP_200_OK
        )


class WorkerCertificateViewSet(viewsets.ModelViewSet):
    queryset = WorkerCertificate.objects.all()
    serializer_class = WorkerCertificateSerializer
    filter_backends = [DjangoFilterBackend]
    filterset_fields = ["worker_id", "station_code", "revoked"]

    @action(detail=True, methods=["post"], url_path="revoke")
    def revoke(self, request, pk=None):
        cert = self.get_object()
        if cert.revoked:
            return Response(
                {"detail": "Already revoked."}, status=status.HTTP_400_BAD_REQUEST
            )
        revoked_by = request.data.get("revoked_by")
        cert.revoked = True
        cert.revoked_at = timezone.now()
        cert.revoked_by = revoked_by
        cert.save()
        return Response(
            WorkerCertificateSerializer(cert).data, status=status.HTTP_200_OK
        )


class TrainingProgressViewSet(viewsets.ModelViewSet):
    queryset = TrainingProgress.objects.all()
    serializer_class = TrainingProgressSerializer
    filter_backends = [DjangoFilterBackend]
    filterset_fields = ["scope", "scope_code"]

class LessonAttemptViewSet(viewsets.ModelViewSet):
    queryset = LessonAttempt.objects.all()
    serializer_class = LessonAttemptSerializer
    filter_backends = [DjangoFilterBackend]
    filterset_fields = ["worker_id","lesson","completed"]

class CertAuditLogViewSet(viewsets.ReadOnlyModelViewSet):
    queryset = CertAuditLog.objects.all()
    serializer_class = CertAuditLogSerializer
