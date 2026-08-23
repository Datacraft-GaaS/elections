import logging
from typing import Any, Protocol, runtime_checkable

from eel_auth.subject import Subject

_log = logging.getLogger(__name__)


@runtime_checkable
class AuditSink(Protocol):
    async def record(
        self,
        subject: Subject,
        action: str,
        resource: str,
        outcome: str,
        metadata: dict[str, Any] | None = None,
    ) -> None: ...


class NoopAuditSink:
    async def record(self, *args: Any, **kwargs: Any) -> None:
        # Deliberately discards every event: this sink exists for services that
        # run without an audit backend.
        return None


class LogAuditSink:
    """Logs audit events to a structured logger."""

    def __init__(self, logger: logging.Logger | None = None) -> None:
        self._logger = logger or logging.getLogger("eel_auth.audit")

    def _log_audit(
        self,
        subject: Subject,
        action: str,
        resource: str,
        outcome: str,
        metadata: dict[str, Any] | None,
    ) -> None:
        self._logger.info(
            "audit sub=%s action=%s resource=%s outcome=%s metadata=%r",
            subject.sub,
            action,
            resource,
            outcome,
            metadata,
        )

    async def record(
        self,
        subject: Subject,
        action: str,
        resource: str,
        outcome: str,
        metadata: dict[str, Any] | None = None,
    ) -> None:
        self._log_audit(subject, action, resource, outcome, metadata)
