"""URL configuration for declaration-admin (WP-3.3)."""

from django.urls import path

from . import views

app_name = "declaration_admin"

urlpatterns = [
    # Declarations queue
    path("declarations/", views.declarations_queue, name="declarations_queue"),
    path(
        "declarations/<str:declaration_id>/",
        views.declaration_detail,
        name="declaration_detail",
    ),
    # Certificate
    path("certificates/", views.certificate_panel, name="certificate_panel"),
    path(
        "certificates/<str:declaration_id>/issue/",
        views.issue_certificate,
        name="issue_certificate",
    ),
    # Petition window
    path(
        "petition-windows/",
        views.petition_window_monitor,
        name="petition_window_monitor",
    ),
    # Correction
    path(
        "declarations/<str:declaration_id>/correct/",
        views.correction_workflow,
        name="correction_workflow",
    ),
    # Reconciliation holds
    path(
        "reconciliation-holds/",
        views.reconciliation_holds_queue,
        name="reconciliation_holds_queue",
    ),
    path(
        "reconciliation-holds/<str:hold_id>/adjudicate/",
        views.discrepancy_adjudication,
        name="discrepancy_adjudication",
    ),
]
