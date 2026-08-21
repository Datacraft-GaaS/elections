"""URL configuration for support-admin (WP-5.5, M25, M27)."""

from django.urls import path

from . import views

app_name = "support_admin"

urlpatterns = [
    # M25: Facility management
    path("facilities/", views.facility_list, name="facility_list"),
    path(
        "facilities/<str:facility_id>/", views.facility_detail, name="facility_detail"
    ),
    # M25: S.44A override queue
    path("s44a/", views.s44a_override_queue, name="s44a_override_queue"),
    # M27: Adjudication catalogue browser
    path(
        "adjudication/catalogue/",
        views.adjudication_catalogue,
        name="adjudication_catalogue",
    ),
    path(
        "adjudication/events/",
        views.adjudication_event_list,
        name="adjudication_event_list",
    ),
    # M27: Rejection rate dashboard
    path(
        "rejection-rate/",
        views.rejection_rate_dashboard,
        name="rejection_rate_dashboard",
    ),
]
