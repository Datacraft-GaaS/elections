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

from accreditation_app.models import (
    AccreditationAuditEntry,
    BadgeRevocationEntry,
    CredentialRevocationEntry,
    IndividualObserver,
    MediaAccreditation,
    MediaOrganisation,
    ObserverOrganisation,
)


@admin.register(MediaOrganisation)
class MediaOrganisationAdmin(admin.ModelAdmin):
    list_display = ["name", "mck_registered", "status", "created_at"]
    search_fields = ["name"]
    list_filter = ["mck_registered", "status"]


@admin.register(MediaAccreditation)
class MediaAccreditationAdmin(admin.ModelAdmin):
    # `applicant_name` and `id_number` are not columns here -- the SQL keeps
    # data class B in a separate encrypted vault -- so the badge serial and the
    # organisation identify the row.
    list_display = ["mck_membership_number", "organisation", "status", "badge_serial"]
    search_fields = ["mck_membership_number", "badge_serial"]
    list_filter = ["status"]
    readonly_fields = ["badge_serial", "badge_issued_at"]


@admin.register(BadgeRevocationEntry)
class BadgeRevocationEntryAdmin(admin.ModelAdmin):
    list_display = ["badge_serial", "reason", "revoked_at", "officer_id"]
    list_filter = ["reason"]
    readonly_fields = ["revoked_at"]


@admin.register(AccreditationAuditEntry)
class AccreditationAuditEntryAdmin(admin.ModelAdmin):
    list_display = ["action", "actor_id", "entity_type", "entity_id", "recorded_at"]
    readonly_fields = ["id", "recorded_at"]


@admin.register(ObserverOrganisation)
class ObserverOrganisationAdmin(admin.ModelAdmin):
    list_display = ["name", "org_type", "nationality", "status", "mou_signed_at"]
    search_fields = ["name"]
    list_filter = ["org_type", "status"]


@admin.register(IndividualObserver)
class IndividualObserverAdmin(admin.ModelAdmin):
    # Same reason as MediaAccreditationAdmin: no `full_name`, no `id_number`.
    list_display = ["credential_serial", "organisation", "status", "nationality"]
    search_fields = ["credential_serial"]
    list_filter = ["status"]


@admin.register(CredentialRevocationEntry)
class CredentialRevocationEntryAdmin(admin.ModelAdmin):
    list_display = ["credential_serial", "revoked_at", "officer_id"]
    readonly_fields = ["revoked_at"]
