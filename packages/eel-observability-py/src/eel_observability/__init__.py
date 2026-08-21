"""eel-observability: OTel init, SLO helpers, audit shadow triggers, and PII log scrubbing."""

from eel_observability.audit_shadow import generate_audit_shadow_trigger
from eel_observability.log_scrubber import PiiScrubberFilter, install_scrubber
from eel_observability.otel import get_meter, get_tracer, init_otel
from eel_observability.slo import SloTimer, record_error, record_latency, record_success

__all__ = [
    "init_otel",
    "get_tracer",
    "get_meter",
    "record_latency",
    "record_error",
    "record_success",
    "SloTimer",
    "generate_audit_shadow_trigger",
    "PiiScrubberFilter",
    "install_scrubber",
]
