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

from django.urls import path

from . import views

app_name = "telemetry"

urlpatterns = [
    path("", views.hard_questions_view, name="hard_questions"),
    path("presets/<str:name>/run/", views.run_preset_view, name="run_preset"),
    path("builder/", views.query_builder_view, name="query_builder"),
    path("builder/run/", views.run_structured_query_view, name="run_structured"),
    path("builder/nl/", views.nl_translate_view, name="nl_translate"),
    path("export/", views.evidence_export_view, name="evidence_export"),
    path(
        "export/<str:query_id>/trigger/",
        views.trigger_export_view,
        name="trigger_export",
    ),
]
