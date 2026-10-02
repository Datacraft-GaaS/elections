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

from __future__ import annotations

from opentelemetry import metrics, trace
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor


def init_otel(
    service_name: str,
    *,
    otlp_endpoint: str = "http://localhost:4317",
    otlp_headers: dict[str, str] | None = None,
    resource_attributes: dict[str, str] | None = None,
) -> tuple[TracerProvider, MeterProvider]:
    """Initialize OTel SDK. Returns (tracer_provider, meter_provider)."""
    attrs: dict[str, str] = {"service.name": service_name}
    if resource_attributes:
        attrs.update(resource_attributes)
    resource = Resource.create(attrs)

    tracer_provider = TracerProvider(resource=resource)
    exporter = OTLPSpanExporter(
        endpoint=otlp_endpoint,
        headers=otlp_headers or {},
    )
    tracer_provider.add_span_processor(BatchSpanProcessor(exporter))
    trace.set_tracer_provider(tracer_provider)

    meter_provider = MeterProvider(resource=resource)
    metrics.set_meter_provider(meter_provider)

    return tracer_provider, meter_provider


def get_tracer(name: str) -> trace.Tracer:
    return trace.get_tracer(name)


def get_meter(name: str) -> metrics.Meter:
    return metrics.get_meter(name)
