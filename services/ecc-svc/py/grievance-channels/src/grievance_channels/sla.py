"""SLA routing and breach detection for grievance cases (WP-4.1)."""

from __future__ import annotations

import logging
from datetime import datetime, timedelta, timezone

from .models import GrievanceCase, SlaPolicy

_log_ = logging.getLogger(__name__)

# Standard SLA policies keyed by (severity,).
# Category overrides can be layered on top in future iterations.
SLA_POLICIES: dict[str, SlaPolicy] = {
    "critical": SlaPolicy(
        category="*", severity="critical", response_hours=2, resolution_hours=24
    ),
    "high": SlaPolicy(
        category="*", severity="high", response_hours=8, resolution_hours=48
    ),
    "medium": SlaPolicy(
        category="*", severity="medium", response_hours=24, resolution_hours=72
    ),
    "low": SlaPolicy(
        category="*", severity="low", response_hours=48, resolution_hours=168
    ),
}


class SlaRouter:
    """Compute SLA deadlines and detect breached cases."""

    def compute_deadline(
        self, category: str, severity: str, filed_at: datetime
    ) -> datetime:
        """Return the resolution deadline for a case given severity."""
        policy = SLA_POLICIES.get(severity, SLA_POLICIES["low"])
        deadline = filed_at + timedelta(hours=policy.resolution_hours)
        _log_.info(
            "sla.compute_deadline severity=%s resolution_hours=%d deadline=%s",
            severity,
            policy.resolution_hours,
            deadline.isoformat(),
        )
        return deadline

    async def check_breaches(self, cases: list[GrievanceCase]) -> list[str]:
        """Return case_ids whose SLA deadline has passed and are still open."""
        now = datetime.now(tz=timezone.utc)
        breached: list[str] = []
        for case in cases:
            if case.status in ("resolved", "closed"):
                continue
            if case.sla_deadline is not None and case.sla_deadline < now:
                _log_.warning(
                    "sla.breach case_id=%s severity=%s", case.case_id, case.severity
                )
                breached.append(case.case_id)
        return breached
