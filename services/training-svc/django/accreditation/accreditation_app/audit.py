"""Audit helpers for accreditation actions."""

import logging
from uuid6 import uuid7

logger = logging.getLogger(__name__)


def uuid7str() -> str:
    return str(uuid7())


def record_audit(
    action: str,
    actor_id: str,
    accreditation_id: str,
    purpose_code: str,
    ip: str | None = None,
    merkle_node_id: str = "",
) -> None:
    """Write an audit entry for an accreditation action."""
    from accreditation_app.models import AccreditationAuditEntry

    AccreditationAuditEntry.objects.create(
        id=uuid7(),
        action=action,
        actor_id=actor_id,
        accreditation_id=accreditation_id,
        purpose_code=purpose_code,
        ip=ip,
        merkle_node_id=merkle_node_id,
    )
    logger.info(
        "audit action=%s actor=%s accreditation=%s", action, actor_id, accreditation_id
    )
