"""Top-level URL conf for the declaration-admin Django project."""

from django.contrib import admin
from django.urls import include, path

urlpatterns = [
    path("admin/", admin.site.urls),
    path("", include("declaration_admin.urls", namespace="declaration_admin")),
]
