from django.contrib import admin
from django.urls import include, path

urlpatterns = [
    path("admin/", admin.site.urls),
    path("authoring/", include("adhoc_editor.apps.authoring.urls")),
]
