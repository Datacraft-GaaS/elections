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

"""CRL (Certificate/Badge Revocation List) generation helpers."""

import logging
from datetime import datetime, timezone


_log = logging.getLogger(__name__)


def generate_crl() -> dict:
    """Return current CRL as {generated_at, revoked_serials}.

    Queries BadgeRevocationEntry (unmanaged, may not exist) and
    CredentialRevocationEntry independently so one missing table
    does not prevent the other from contributing to the CRL.
    """
    all_serials: list[str] = []

    try:
        from accreditation_app.models import BadgeRevocationEntry

        all_serials.extend(
            str(s)
            for s in BadgeRevocationEntry.objects.values_list("badge_serial", flat=True)
        )
    except Exception as exc:
        _log.warning("BadgeRevocationEntry unavailable, skipping badge serials: %s", exc)

    try:
        from accreditation_app.models import CredentialRevocationEntry

        all_serials.extend(
            str(s)
            for s in CredentialRevocationEntry.objects.values_list(
                "credential_serial", flat=True
            )
        )
    except Exception as exc:
        _log.warning(
            "CredentialRevocationEntry unavailable, skipping credential serials: %s",
            exc,
        )

    return {
        "generated_at": datetime.now(tz=timezone.utc).isoformat(),
        "revoked_serials": all_serials,
    }
