from django.contrib import admin
from django.urls import include, path

urlpatterns = [
    path("admin/", admin.site.urls),
    path("adjudication/", include("abis_console.apps.adjudication.urls")),
]
