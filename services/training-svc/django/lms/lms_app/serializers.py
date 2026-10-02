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

from rest_framework import serializers

from .models import CourseModule, TrainingProgress, WorkerCertificate, WorkerEnrollment


class CourseModuleSerializer(serializers.ModelSerializer):
    class Meta:
        model = CourseModule
        fields = "__all__"
        read_only_fields = ["id"]


class WorkerEnrollmentSerializer(serializers.ModelSerializer):
    class Meta:
        model = WorkerEnrollment
        fields = "__all__"
        read_only_fields = [
            "id",
            "attempts",
            "best_score_pct",
            "certified",
            "completed_at",
        ]


class AssessmentSubmitSerializer(serializers.Serializer):
    score_pct = serializers.DecimalField(max_digits=5, decimal_places=2)


class WorkerCertificateSerializer(serializers.ModelSerializer):
    class Meta:
        model = WorkerCertificate
        fields = "__all__"
        read_only_fields = [
            "id",
            "issued_at",
            "certificate_hash",
            "qr_verification_url",
        ]


class TrainingProgressSerializer(serializers.ModelSerializer):
    class Meta:
        model = TrainingProgress
        fields = "__all__"
        read_only_fields = ["id", "snapshot_at"]

from .models import Course, Lesson, LessonAttempt, CertAuditLog

class CourseSerializer(serializers.ModelSerializer):
    class Meta: model=Course; fields="__all__"

class LessonSerializer(serializers.ModelSerializer):
    class Meta: model=Lesson; fields="__all__"

class LessonAttemptSerializer(serializers.ModelSerializer):
    class Meta: model=LessonAttempt; fields="__all__"; read_only_fields=["id","attempted_at"]

class CertAuditLogSerializer(serializers.ModelSerializer):
    class Meta: model=CertAuditLog; fields="__all__"; read_only_fields=["id","recorded_at"]
