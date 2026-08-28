import logging

_logger = logging.getLogger(__name__)


class AuditLogger:
    """Structured audit logger for governance admin actions.

    Emits a single structured record per action. Deployments can attach a
    handler to the ``governance_app.audit`` logger to persist records to the
    audit shadow store.
    """

    def log_action(
        self, actor: str, action: str, target_id: str, detail: str = ""
    ) -> None:
        _logger.info(
            "audit actor=%s action=%s target_id=%s detail=%s",
            actor,
            action,
            target_id,
            detail,
            extra={
                "audit_event": {
                    "actor": actor,
                    "action": action,
                    "target_id": target_id,
                    "detail": detail,
                }
            },
        )
