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

from uuid6 import uuid7


def uuid7str() -> str:
    return str(uuid7())


def generate_badge_serial() -> str:
    """Generate a new unique badge serial (UUID7)."""
    return uuid7str()


def verify_badge(serial: str, crl: dict) -> str:
    """Verify badge status against CRL.

    Returns 'ACTIVE', 'REVOKED', or 'SUSPENDED'.
    """
    revoked_serials: list[str] = crl.get("revoked_serials", [])
    if serial in revoked_serials:
        return "REVOKED"

    try:
        from accreditation_app.models import MediaAccreditation

        accred = MediaAccreditation.objects.filter(badge_serial=serial).first()
        if accred is None:
            return "REVOKED"
        if accred.status == "SUSPENDED":
            return "SUSPENDED"
        if accred.status in ("ACTIVE", "BADGE_ISSUED"):
            return "ACTIVE"
        return "REVOKED"
    except Exception:
        return "ACTIVE"
