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

from starlette.requests import Request
from starlette.testclient import TestClient
from starlette.applications import Starlette
from starlette.responses import PlainTextResponse
from starlette.routing import Route

from eel_loadshedding.middleware import (
    LoadSheddingMiddleware,
    Priority,
    default_priority_extractor,
    shed_response,
)


def test_shed_response_status_503():
    resp = shed_response(retry_after_s=10)
    assert resp.status_code == 503


def test_shed_response_content_type():
    resp = shed_response()
    assert resp.media_type == "application/problem+json"


def test_shed_response_retry_after_header():
    resp = shed_response(retry_after_s=30)
    assert resp.headers["Retry-After"] == "30"


def test_shed_response_body_has_required_fields():
    import json

    resp = shed_response()
    body = json.loads(resp.body)
    assert body["status"] == 503
    assert "title" in body
    assert "detail" in body


def _make_request(path: str = "/", headers: dict[str, str] | None = None) -> Request:
    scope = {
        "type": "http",
        "method": "GET",
        "path": path,
        "query_string": b"",
        "headers": [
            (k.lower().encode(), v.encode()) for k, v in (headers or {}).items()
        ],
    }
    return Request(scope)


def test_default_priority_health_is_critical():
    req = _make_request("/health")
    assert default_priority_extractor(req) == Priority.CRITICAL


def test_default_priority_health_subpath():
    req = _make_request("/health/ready")
    assert default_priority_extractor(req) == Priority.CRITICAL


def test_default_priority_header_high():
    req = _make_request("/api/v1/data", {"X-Priority": "high"})
    assert default_priority_extractor(req) == Priority.HIGH


def test_default_priority_header_low():
    req = _make_request("/api/v1/data", {"X-Priority": "low"})
    assert default_priority_extractor(req) == Priority.LOW


def test_default_priority_no_header_is_normal():
    req = _make_request("/api/v1/data")
    assert default_priority_extractor(req) == Priority.NORMAL


def test_default_priority_header_critical():
    req = _make_request("/api/v1/data", {"X-Priority": "critical"})
    assert default_priority_extractor(req) == Priority.CRITICAL


def _make_app(max_queue_depth: int = 100) -> TestClient:
    async def homepage(request: Request) -> PlainTextResponse:
        return PlainTextResponse("ok")

    app = Starlette(routes=[Route("/", homepage), Route("/health", homepage)])
    app.add_middleware(LoadSheddingMiddleware, max_queue_depth=max_queue_depth)
    return TestClient(app, raise_server_exceptions=False)


def test_loadshedding_passes_through_normal():
    client = _make_app(max_queue_depth=100)
    resp = client.get("/")
    assert resp.status_code == 200
    assert resp.text == "ok"


def test_loadshedding_health_always_passes():
    client = _make_app(max_queue_depth=100)
    resp = client.get("/health")
    assert resp.status_code == 200
