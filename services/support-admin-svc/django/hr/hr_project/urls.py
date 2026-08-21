"""Root URL configuration for eel-hr (WP-5.6)."""

from django.urls import include, path

urlpatterns = [
    path("", include("hr_app.urls")),
]
