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

app_name = "reports"

urlpatterns = [
    path("runs/new/", views.new_run, name="new_run"),
    path("runs/<str:run_id>/", views.run_detail, name="run_detail"),
    path("deliveries/", views.delivery_dashboard, name="delivery_dashboard"),
    path(
        "deliveries/<str:delivery_id>/retry/",
        views.retry_delivery,
        name="retry_delivery",
    ),
    path("subscriptions/new/", views.subscription_wizard, name="subscription_wizard"),
    path(
        "artifacts/<str:artifact_id>/download/",
        views.artifact_download,
        name="artifact_download",
    ),
]
