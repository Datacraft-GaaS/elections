from django.urls import path

from . import views

app_name = "reports"

urlpatterns = [
    path("runs/new/", views.new_run, name="new_run"),
    path("runs/<str:run_id>/", views.run_detail, name="run_detail"),
    path("deliveries/", views.delivery_dashboard, name="delivery_dashboard"),
    path(
        "deliveries/<str:delivery_id>/retry/",
        views.retry_delivery,
        name="retry_delivery",
    ),
    path("subscriptions/new/", views.subscription_wizard, name="subscription_wizard"),
    path(
        "artifacts/<str:artifact_id>/download/",
        views.artifact_download,
        name="artifact_download",
    ),
]
