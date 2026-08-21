"""Top-level URL conf for support-admin Django project."""

from django.contrib import admin
from django.urls import include, path

urlpatterns = [
    path("admin/", admin.site.urls),
    path("", include("support_admin.urls", namespace="support_admin")),
]
