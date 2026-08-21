from django.contrib import admin
from django.utils import timezone

from .models import CourseModule, TrainingProgress, WorkerCertificate, WorkerEnrollment


@admin.register(CourseModule)
class CourseModuleAdmin(admin.ModelAdmin):
    list_display = [
        "title",
        "target_audience",
        "module_type",
        "language",
        "pass_mark_pct",
        "published_at",
    ]
    list_filter = ["module_type", "target_audience", "language"]
    search_fields = ["title"]
    readonly_fields = ["id"]


@admin.register(WorkerEnrollment)
class WorkerEnrollmentAdmin(admin.ModelAdmin):
    list_display = [
        "worker_id",
        "module",
        "status",
        "attempts",
        "best_score_pct",
        "certified",
    ]
    list_filter = ["status", "certified"]
    search_fields = ["worker_id"]
    readonly_fields = ["id"]


@admin.register(WorkerCertificate)
class WorkerCertificateAdmin(admin.ModelAdmin):
    list_display = ["worker_id", "station_code", "issued_at", "expires_at", "revoked"]
    list_filter = ["revoked"]
    search_fields = ["worker_id", "station_code"]
    readonly_fields = ["id", "issued_at", "certificate_hash"]
    actions = ["revoke_certificates"]

    @admin.action(description="Revoke selected certificates")
    def revoke_certificates(self, request, queryset):
        updated = queryset.filter(revoked=False).update(
            revoked=True,
            revoked_at=timezone.now(),
        )
        self.message_user(request, f"{updated} certificate(s) revoked.")


@admin.register(TrainingProgress)
class TrainingProgressAdmin(admin.ModelAdmin):
    list_display = [
        "scope",
        "scope_code",
        "snapshot_at",
        "total_workers",
        "certified",
        "certification_rate_pct",
    ]
    list_filter = ["scope"]
    readonly_fields = ["id", "snapshot_at"]
