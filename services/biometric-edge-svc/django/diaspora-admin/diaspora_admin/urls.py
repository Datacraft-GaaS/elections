"""URL configuration for diaspora-admin."""

from django.contrib import admin
from django.urls import path, include

from diaspora_admin.apps.embassies import views as embassy_views

urlpatterns = [
    path("admin/", admin.site.urls),
    path(
        "embassies/import/", embassy_views.ImportRollView.as_view(), name="import-roll"
    ),
]
