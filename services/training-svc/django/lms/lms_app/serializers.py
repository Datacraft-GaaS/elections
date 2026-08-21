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
