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

"""DCI/ODPP referral emitters (tier-B integration stubs) (WP-4.1)."""

from __future__ import annotations

import logging

from .models import ReferralPackage

_log_ = logging.getLogger(__name__)


def uuid7str() -> str:
    from uuid6 import uuid7

    return str(uuid7())


class ReferralEmitter:
    """Emit referral packages to DCI and ODPP (stubs pending tier-B integration)."""

    async def refer_dci(self, case_id: str, evidence_pkg_id: str) -> ReferralPackage:
        """Stub: queue a referral to the Directorate of Criminal Investigations."""
        assert case_id, "case_id required"
        assert evidence_pkg_id, "evidence_pkg_id required"
        pkg = ReferralPackage(
            case_id=case_id,
            referral_type="dci",
            evidence_pkg_id=evidence_pkg_id,
            submitted_at=None,  # set to now when integration is live
        )
        _log_.info(
            "referral.dci case_id=%s evidence_pkg_id=%s (stub: queued, not yet submitted)",
            case_id,
            evidence_pkg_id,
        )
        return pkg

    async def refer_odpp(self, case_id: str, evidence_pkg_id: str) -> ReferralPackage:
        """Stub: queue a referral to the Office of the Director of Public Prosecutions."""
        assert case_id, "case_id required"
        assert evidence_pkg_id, "evidence_pkg_id required"
        pkg = ReferralPackage(
            case_id=case_id,
            referral_type="odpp",
            evidence_pkg_id=evidence_pkg_id,
            submitted_at=None,
        )
        _log_.info(
            "referral.odpp case_id=%s evidence_pkg_id=%s (stub: queued, not yet submitted)",
            case_id,
            evidence_pkg_id,
        )
        return pkg
