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

from django.contrib import admin

from .models import (
    AdhocPreview,
    AdhocReport,
    AdhocReportVersion,
    AdhocShare,
    AdhocSnippet,
)


@admin.register(AdhocReport)
class AdhocReportAdmin(admin.ModelAdmin):
    list_display = [
        "title",
        "slug",
        "author_id",
        "visibility",
        "data_class",
        "updated_at",
        "archived_at",
    ]
    list_filter = ["visibility", "data_class", "archived_at"]
    search_fields = ["title", "slug", "description"]
    readonly_fields = ["adhoc_id", "created_at", "updated_at"]
    ordering = ["-updated_at"]

    def get_queryset(self, request):
        qs = super().get_queryset(request)
        if not request.user.is_superuser:
            qs = qs.filter(author_id=str(request.user.pk))
        return qs


@admin.register(AdhocReportVersion)
class AdhocReportVersionAdmin(admin.ModelAdmin):
    list_display = [
        "version_id",
        "adhoc",
        "version_number",
        "source_type",
        "safety_score",
        "created_at",
    ]
    list_filter = ["source_type"]
    search_fields = ["commit_message", "source"]
    readonly_fields = ["version_id", "created_at", "linted_at"]
    ordering = ["-created_at"]

    def get_queryset(self, request):
        qs = super().get_queryset(request)
        if not request.user.is_superuser:
            qs = qs.filter(adhoc__author_id=str(request.user.pk))
        return qs


@admin.register(AdhocPreview)
class AdhocPreviewAdmin(admin.ModelAdmin):
    list_display = [
        "preview_id",
        "version",
        "preview_format",
        "sample_limit",
        "rows_returned",
        "expires_at",
    ]
    list_filter = ["preview_format"]
    readonly_fields = ["preview_id", "requested_at", "expires_at"]
    ordering = ["-requested_at"]

    def get_queryset(self, request):
        qs = super().get_queryset(request)
        if not request.user.is_superuser:
            qs = qs.filter(requested_by=str(request.user.pk))
        return qs


@admin.register(AdhocShare)
class AdhocShareAdmin(admin.ModelAdmin):
    list_display = [
        "share_id",
        "adhoc",
        "shared_with",
        "permission",
        "granted_by",
        "granted_at",
        "revoked_at",
    ]
    list_filter = ["permission"]
    readonly_fields = ["share_id", "granted_at"]
    ordering = ["-granted_at"]

    def get_queryset(self, request):
        qs = super().get_queryset(request)
        if not request.user.is_superuser:
            qs = qs.filter(granted_by=str(request.user.pk))
        return qs


@admin.register(AdhocSnippet)
class AdhocSnippetAdmin(admin.ModelAdmin):
    list_display = ["name", "kind", "visibility", "author_id", "updated_at"]
    list_filter = ["kind", "visibility"]
    search_fields = ["name", "body"]
    readonly_fields = ["snippet_id", "created_at", "updated_at"]
    ordering = ["name"]

    def get_queryset(self, request):
        qs = super().get_queryset(request)
        if not request.user.is_superuser:
            qs = qs.filter(author_id=str(request.user.pk))
        return qs
