"""Stub doubles for KRCS/MoH/NDMU dispatch with circuit-breaker pattern."""

import logging

logger = logging.getLogger(__name__)


def dispatch_krcs(incident_id: str, eta: int | None = None) -> dict:
    """Dispatch to Kenya Red Cross Society. Returns stub ack."""
    try:
        logger.info("KRCS dispatch incident=%s eta=%s", incident_id, eta)
        return {
            "status": "dispatched",
            "ack": "stub",
            "provider": "KRCS",
            "incident_id": incident_id,
        }
    except Exception as exc:
        logger.error("KRCS dispatch failed incident=%s error=%s", incident_id, exc)
        return {
            "status": "error",
            "ack": "fallback",
            "provider": "KRCS",
            "incident_id": incident_id,
        }


def dispatch_moh(incident_id: str, eta: int | None = None) -> dict:
    """Dispatch to Ministry of Health. Returns stub ack."""
    try:
        logger.info("MoH dispatch incident=%s eta=%s", incident_id, eta)
        return {
            "status": "dispatched",
            "ack": "stub",
            "provider": "MoH",
            "incident_id": incident_id,
        }
    except Exception as exc:
        logger.error("MoH dispatch failed incident=%s error=%s", incident_id, exc)
        return {
            "status": "error",
            "ack": "fallback",
            "provider": "MoH",
            "incident_id": incident_id,
        }


def dispatch_ndmu(incident_id: str, eta: int | None = None) -> dict:
    """Dispatch to National Disaster Management Unit. Returns stub ack."""
    try:
        logger.info("NDMU dispatch incident=%s eta=%s", incident_id, eta)
        return {
            "status": "dispatched",
            "ack": "stub",
            "provider": "NDMU",
            "incident_id": incident_id,
        }
    except Exception as exc:
        logger.error("NDMU dispatch failed incident=%s error=%s", incident_id, exc)
        return {
            "status": "error",
            "ack": "fallback",
            "provider": "NDMU",
            "incident_id": incident_id,
        }


_DISPATCH_MAP = {
    "KRCS": dispatch_krcs,
    "MoH": dispatch_moh,
    "NDMU": dispatch_ndmu,
}


def dispatch(provider: str, incident_id: str, eta: int | None = None) -> dict:
    """Route dispatch to the correct provider stub."""
    fn = _DISPATCH_MAP.get(provider)
    if fn is None:
        raise ValueError(f"Unknown dispatch provider: {provider}")
    return fn(incident_id, eta)
