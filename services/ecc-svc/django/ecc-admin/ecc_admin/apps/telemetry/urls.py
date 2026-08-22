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
