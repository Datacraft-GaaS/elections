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

"""Badge PDF and QR helpers."""

import logging

from uuid6 import uuid7

_log = logging.getLogger(__name__)


def uuid7str() -> str:
    return str(uuid7())


def generate_badge_serial() -> str:
    """Generate a new unique badge serial (UUID7)."""
    return uuid7str()


def verify_badge(serial: str, crl: dict) -> str:
    """Verify badge status against the CRL and the badge's own row.

    Returns 'ACTIVE', 'REVOKED', or 'SUSPENDED'.

    An unrecognised serial is REVOKED, and so is a lookup that raises: a gate
    that cannot confirm a badge is valid must not report that it is. The
    previous version returned ACTIVE from the `except`, so a database outage
    verified every badge in the system. A badge absent from the table is not a
    valid badge -- it is one whose revocation, or whose non-issuance, cannot be
    ruled out.
    """
    revoked_serials: list[str] = crl.get("revoked_serials", [])
    if serial in revoked_serials:
        return "REVOKED"

    try:
        from accreditation_app.models import MediaAccreditation

        accred = MediaAccreditation.objects.filter(badge_serial=serial).first()
    except Exception as exc:
        _log.warning("badge verify lookup failed for %s: %s", serial, exc)
        return "REVOKED"

    if accred is None:
        return "REVOKED"
    if accred.status == "suspended":
        return "SUSPENDED"
    if accred.status in ("approved", "badge_issued"):
        return "ACTIVE"
    return "REVOKED"
