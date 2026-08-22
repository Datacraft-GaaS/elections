from django.urls import path

from . import views

app_name = "rfid_covers"

urlpatterns = [
    path("rfid/box-chain/", views.box_chain_view, name="box_chain"),
    path("rfid/tamper-triage/", views.tamper_triage_view, name="tamper_triage"),
    path("rfid/reader-health/", views.reader_health_view, name="reader_health"),
]
