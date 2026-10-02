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

"""ABIS vendor worker - processes match requests and publishes results."""

from __future__ import annotations
import asyncio
from .adapter import LegacyAbisAdapter
from .models import MatchRequest, MatchResult, VendorConfig
import structlog

_log = structlog.get_logger()


class AbisVendorWorker:
    """Worker that pulls match requests and delegates to the legacy ABIS adapter."""

    def __init__(self, config: VendorConfig) -> None:
        self._config = config

    async def process_one(self, request: MatchRequest) -> MatchResult:
        """Process a single match request via the legacy adapter."""
        async with LegacyAbisAdapter(self._config) as adapter:
            result = await adapter.submit_match(request)
        _log.info(
            "abis_vendor.match_processed",
            request_id=request.request_id,
            outcome=result.outcome,
        )
        return result

    async def run_batch(self, requests: list[MatchRequest]) -> list[MatchResult]:
        """Process a batch of match requests concurrently (max 10 parallel)."""
        sem = asyncio.Semaphore(10)

        async def bounded(req: MatchRequest) -> MatchResult:
            async with sem:
                return await self.process_one(req)

        return list(await asyncio.gather(*[bounded(r) for r in requests]))
