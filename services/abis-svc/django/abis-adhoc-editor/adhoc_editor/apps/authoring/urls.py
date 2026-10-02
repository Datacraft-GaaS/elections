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

urlpatterns = [
    path("", views.report_list, name="report_list"),
    path("reports/<str:adhoc_id>/", views.report_editor, name="report_editor"),
    path(
        "reports/<str:adhoc_id>/save/",
        views.htmx_save_version,
        name="htmx_save_version",
    ),
    path("versions/<str:version_id>/preview/", views.htmx_preview, name="htmx_preview"),
    path(
        "versions/<str:version_id>/publish/",
        views.publish_wizard,
        name="publish_wizard",
    ),
]
