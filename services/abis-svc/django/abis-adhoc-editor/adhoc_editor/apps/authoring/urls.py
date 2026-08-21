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
