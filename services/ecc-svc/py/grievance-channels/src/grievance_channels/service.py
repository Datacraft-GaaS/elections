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

"""Main GrievanceService orchestrating case lifecycle (WP-4.1)."""

from __future__ import annotations

import hashlib
import logging
from datetime import datetime, timezone

from .evidence import EvidencePackager
from .models import EvidencePackage, GrievanceCase, ReferralPackage
from .referrals import ReferralEmitter
from .sla import SlaRouter

_log_ = logging.getLogger(__name__)


def uuid7str() -> str:
    from uuid6 import uuid7

    return str(uuid7())


class GrievanceService:
    """Lifecycle management for grievance cases across all channels."""

    def __init__(self) -> None:
        self._sla = SlaRouter()
        self._packager = EvidencePackager()
        self._referrals = ReferralEmitter()
        # In-process store: replace with DB-backed repo in production.
        self._cases: dict[str, GrievanceCase] = {}

    async def file_case(
        self,
        channel: str,
        category: str,
        severity: str,
        description: str,
    ) -> GrievanceCase:
        assert channel in ("ussd", "sms", "whatsapp", "web", "kiosk", "api"), (
            "invalid channel"
        )
        assert severity in ("low", "medium", "high", "critical"), "invalid severity"

        now = datetime.now(tz=timezone.utc)
        description_hash = hashlib.sha256(description.encode()).hexdigest()
        sla_deadline = self._sla.compute_deadline(category, severity, now)
        case = GrievanceCase(
            case_id=uuid7str(),
            channel=channel,  # type: ignore[arg-type]
            category=category,
            severity=severity,  # type: ignore[arg-type]
            description_hash=description_hash,
            sla_deadline=sla_deadline,
            created_at=now,
        )
        self._cases[case.case_id] = case
        _log_.info(
            "grievance.filed case_id=%s channel=%s severity=%s event=grievance_filed.v1",
            case.case_id,
            channel,
            severity,
        )
        return case

    async def assign_case(self, case_id: str, assignee: str) -> GrievanceCase:
        assert case_id in self._cases, f"case {case_id} not found"
        case = self._cases[case_id]
        updated = case.model_copy(
            update={"status": "assigned", "assigned_to": assignee}
        )
        self._cases[case_id] = updated
        _log_.info(
            "grievance.assigned case_id=%s assignee=%s event=grievance_assigned.v1",
            case_id,
            assignee,
        )
        return updated

    async def escalate_case(self, case_id: str, level: int) -> GrievanceCase:
        assert case_id in self._cases, f"case {case_id} not found"
        assert 1 <= level <= 4, "escalation level must be 1-4"
        case = self._cases[case_id]
        updated = case.model_copy(update={"status": "escalated"})
        self._cases[case_id] = updated
        _log_.info("grievance.escalated case_id=%s level=%d", case_id, level)
        return updated

    async def assemble_evidence(
        self, case_id: str, content_refs: list[str]
    ) -> EvidencePackage:
        pkg = await self._packager.assemble(case_id, content_refs)
        _log_.info(
            "grievance.evidence_packaged pkg_id=%s case_id=%s event=evidence_packaged.v1",
            pkg.pkg_id,
            case_id,
        )
        return pkg

    async def refer_to_authority(self, case_id: str, authority: str) -> ReferralPackage:
        assert authority in ("dci", "odpp"), "authority must be dci or odpp"
        # Assemble evidence from case refs for referral.
        case = self._cases.get(case_id)
        refs = case.evidence_refs if case else ["stub-ref"]
        pkg = await self._packager.assemble(case_id, refs or ["stub-ref"])
        if authority == "dci":
            return await self._referrals.refer_dci(case_id, pkg.pkg_id)
        return await self._referrals.refer_odpp(case_id, pkg.pkg_id)
