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

"""eel_web conformance tests."""

import django
from django.conf import settings


def setup_django() -> None:
    if not settings.configured:
        settings.configure(
            INSTALLED_APPS=[
                "django.contrib.contenttypes",
                "django.contrib.auth",
                "eel_web",
            ],
            DATABASES={
                "default": {
                    "ENGINE": "django.db.backends.sqlite3",
                    "NAME": ":memory:",
                }
            },
            DEFAULT_AUTO_FIELD="django.db.models.BigAutoField",
        )
        django.setup()


setup_django()


def test_apps_config() -> None:
    from eel_web.apps import EelWebConfig

    assert EelWebConfig.name == "eel_web"


def test_spicedb_backend_has_perm() -> None:
    from eel_web.backends import SpiceDBBackend

    backend = SpiceDBBackend()
    assert backend.has_perm(None, "form34a:view") is False


def test_pii_form_fields() -> None:
    from eel_web.forms import PIIRestrictedField, PIISensitiveField

    assert PIIRestrictedField.pii_tier == "A"
    assert PIISensitiveField.pii_tier == "B"


def test_middleware_classes_exist() -> None:
    from eel_web.middleware import (
        EelAuditMiddleware,
        EelAuthMiddleware,
        EelPiiScrubberMiddleware,
    )

    assert EelAuthMiddleware is not None
    assert EelAuditMiddleware is not None
    assert EelPiiScrubberMiddleware is not None


def test_django_check() -> None:
    from django.core.management import call_command

    call_command("check", "--database", "default")
