"""Signal stubs for eel.training medical events (M60)."""

import logging

logger = logging.getLogger(__name__)

MEDICAL_INCIDENT_REPORTED = "eel.training.medical_incident_reported.v1"
MEDICAL_CLUSTER_ALERT = "eel.training.medical_cluster_alert.v1"
DISPATCH_REQUESTED = "eel.training.dispatch_requested.v1"


def emit_stub(topic: str, payload: dict) -> None:
    """Stub emitter: logs the event. Replace with real bus in production."""
    logger.info("EVENT topic=%s payload=%s", topic, payload)


def on_incident_reported(incident_id: str, station_id: str, severity: str) -> None:
    emit_stub(
        MEDICAL_INCIDENT_REPORTED,
        {
            "incident_id": incident_id,
            "station_id": station_id,
            "severity": severity,
        },
    )


def on_cluster_alert(alert_id: str, county_code: str, incident_count: int) -> None:
    emit_stub(
        MEDICAL_CLUSTER_ALERT,
        {
            "alert_id": alert_id,
            "county_code": county_code,
            "incident_count": incident_count,
        },
    )


def on_dispatch_requested(dispatch_id: str, incident_id: str, provider: str) -> None:
    emit_stub(
        DISPATCH_REQUESTED,
        {
            "dispatch_id": dispatch_id,
            "incident_id": incident_id,
            "provider": provider,
        },
    )
