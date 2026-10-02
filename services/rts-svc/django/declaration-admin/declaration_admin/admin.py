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

"""Django admin registration for declaration-admin (WP-3.3)."""

from django.contrib import admin

from .models import (
    CertificateOfElection,
    Declaration,
    DeclarationAmendment,
    PetitionWindow,
    ReconciliationHold,
)


@admin.register(Declaration)
class DeclarationAdmin(admin.ModelAdmin):
    list_display = (
        "id",
        "contest_id",
        "election_id",
        "status",
        "winner_id",
        "declared_at",
    )
    list_filter = ("status", "election_id")
    search_fields = ("contest_id", "winner_id", "election_id")
    readonly_fields = ("id", "created_at", "updated_at")


@admin.register(CertificateOfElection)
class CertificateAdmin(admin.ModelAdmin):
    list_display = ("id", "declaration", "winner_id", "status", "issued_at")
    list_filter = ("status",)
    readonly_fields = ("id", "created_at")


@admin.register(PetitionWindow)
class PetitionWindowAdmin(admin.ModelAdmin):
    list_display = ("id", "declaration", "status", "window_open_at", "window_close_at")
    list_filter = ("status",)
    readonly_fields = ("id", "created_at")


@admin.register(DeclarationAmendment)
class DeclarationAmendmentAdmin(admin.ModelAdmin):
    list_display = ("id", "declaration", "amended_by", "approved_by", "amended_at")
    readonly_fields = ("id", "created_at")

    # No delete permission: retained 7 years (AC-M51-17).
    def has_delete_permission(self, request, obj=None):
        return False


@admin.register(ReconciliationHold)
class ReconciliationHoldAdmin(admin.ModelAdmin):
    list_display = (
        "id",
        "station_code",
        "result_hold_active",
        "raised_at",
        "resolved_at",
    )
    list_filter = ("result_hold_active",)
    search_fields = ("station_code",)
    readonly_fields = ("id", "created_at")
