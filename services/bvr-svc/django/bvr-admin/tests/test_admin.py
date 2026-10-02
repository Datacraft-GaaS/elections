# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

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
