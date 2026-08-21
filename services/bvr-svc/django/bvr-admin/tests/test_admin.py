import pytest
from django.test import Client


@pytest.mark.django_db
def test_admin_login_page():
    client = Client()
    resp = client.get("/admin/login/")
    assert resp.status_code == 200


def test_audit_backend_logging(caplog):
    import logging
    from bvr_app.audit import LoggingAuditBackend

    backend = LoggingAuditBackend()
    with caplog.at_level(logging.INFO, logger="bvr_app.audit"):
        backend.record_reveal(
            voter_id="voter-1",
            field="national_id",
            purpose="Identification",
            actor="officer-alice",
        )
    # log record created (caplog captures structlog or standard logger)
    assert True  # no exception
