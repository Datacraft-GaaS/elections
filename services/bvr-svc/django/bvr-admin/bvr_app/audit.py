"""Audit backends for PII reveal logging (ADR-0016)."""

import logging
from typing import Protocol

_log = logging.getLogger(__name__)


class AuditBackend(Protocol):
    def record_reveal(
        self, voter_id: str, field: str, purpose: str, actor: str
    ) -> None: ...


class LoggingAuditBackend:
    """Writes PII reveal events to structured log (dev default)."""

    def record_reveal(
        self, voter_id: str, field: str, purpose: str, actor: str
    ) -> None:
        _log.info(
            "pii_reveal",
            extra={
                "voter_id": voter_id,
                "field": field,
                "purpose": purpose,
                "actor": actor,
            },
        )
