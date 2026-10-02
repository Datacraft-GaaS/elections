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

from .models import Kit


@admin.register(Kit)
class KitAdmin(admin.ModelAdmin):
    list_display = (
        "kit_id",
        "serial",
        "kit_type",
        "status",
        "station_code",
        "last_seen_at",
    )
    list_filter = ("status", "kit_type")
    search_fields = ("kit_id", "serial", "station_code")
    readonly_fields = ("kit_id", "serial", "last_seen_at", "created_at")
    ordering = ("-last_seen_at",)
