from django.urls import path
from . import views

urlpatterns = [
    path(
        "bvr/registers/<str:scope>/freeze",
        views.RegisterFreezeView.as_view(),
        name="register_freeze",
    ),
    path(
        "bvr/exports/<str:export_id>/approve",
        views.BulkExportApprovalView.as_view(),
        name="bulk_export_approve",
    ),
]
