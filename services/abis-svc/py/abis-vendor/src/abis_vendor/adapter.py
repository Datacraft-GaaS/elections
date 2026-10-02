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

"""Legacy ABIS adapter - wraps integrations/legacy-abis HTTP endpoint."""

from __future__ import annotations
from datetime import datetime, timezone
import httpx
from .models import MatchOutcome, MatchRequest, MatchResult, VendorConfig


class LegacyAbisAdapter:
    """Adapter for the legacy the Commission ABIS system via integrations/legacy-abis."""

    def __init__(self, config: VendorConfig) -> None:
        self._config = config
        self._client: httpx.AsyncClient | None = None

    async def __aenter__(self) -> LegacyAbisAdapter:
        self._client = httpx.AsyncClient(
            base_url=self._config.adapter_url,
            timeout=self._config.timeout_s,
        )
        return self

    async def __aexit__(self, *_: object) -> None:
        if self._client:
            await self._client.aclose()

    async def submit_match(self, request: MatchRequest) -> MatchResult:
        """Submit a match request to the legacy ABIS system."""
        if self._config.mock_mode:
            return self._mock_result(request)
        assert self._client is not None
        resp = await self._client.post(
            "/v1/match",
            json={
                "request_id": request.request_id,
                "probe_ref": request.probe_ref,
                "modality": request.modality,
            },
        )
        resp.raise_for_status()
        data = resp.json()
        return MatchResult(
            request_id=request.request_id,
            outcome=MatchOutcome(data["outcome"]),
            confidence_score=float(data["score"]),
            vendor_ref=data.get("vendor_ref"),
            processed_at=datetime.now(timezone.utc).isoformat(),
        )

    def _mock_result(self, request: MatchRequest) -> MatchResult:
        """Deterministic mock result for dev/test."""
        h = hash(request.probe_ref) % 1000
        score = (h % 100) / 100.0
        outcome = MatchOutcome.match if score > 0.8 else MatchOutcome.no_match
        return MatchResult(
            request_id=request.request_id,
            outcome=outcome,
            confidence_score=score,
            vendor_ref="mock",
            processed_at=datetime.now(timezone.utc).isoformat(),
        )
