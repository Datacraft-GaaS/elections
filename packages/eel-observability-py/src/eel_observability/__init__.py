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
