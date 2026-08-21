import logging

_logger = logging.getLogger(__name__)


class AuditLogger:
    """Stub audit logger for governance actions."""

    def log_action(
        self, actor: str, action: str, target_id: str, detail: str = ""
    ) -> None:
        msg = f"AUDIT actor={actor} action={action} target={target_id} detail={detail}"
        print(msg)
        _logger.info(msg)
