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

"""Async M-Pesa B2C client (WP-5.6).

Supports sandbox and production Daraja endpoints.
Idempotent per batch_id: a batch already submitted is not resubmitted.
"""

import logging

import httpx

from .models import BatchRequest, BatchResult, DisbursementItem, ItemResult

logger = logging.getLogger(__name__)

_SANDBOX_URL = "https://sandbox.safaricom.co.ke"
_PROD_URL = "https://api.safaricom.co.ke"

_BATCH_ENDPOINT = "/v1/b2c/batch"


class B2CClient:
    """Async M-Pesa B2C disbursement client.

    Parameters
    ----------
    consumer_key:
            Daraja API consumer key.
    consumer_secret:
            Daraja API consumer secret.
    sandbox:
            Use the sandbox endpoint when True (default), production otherwise.
    """

    def __init__(
        self,
        consumer_key: str,
        consumer_secret: str,
        *,
        sandbox: bool = True,
        base_url: str | None = None,
    ) -> None:
        self._consumer_key = consumer_key
        self._consumer_secret = consumer_secret
        self._base_url = base_url or (_SANDBOX_URL if sandbox else _PROD_URL)
        self._submitted: set[str] = set()

    async def disburse_batch(
        self,
        batch_id: str,
        items: list[DisbursementItem],
    ) -> BatchResult:
        """Submit a B2C disbursement batch.

        Idempotent: if batch_id was already submitted this instance returns
        a BatchResult indicating all items as 'already_submitted' without
        making a network call.

        Parameters
        ----------
        batch_id:
                Stable identifier for this batch (use uuid7str() at call site).
        items:
                List of DisbursementItem to disburse.

        Returns
        -------
        BatchResult
        """
        assert batch_id, "batch_id must not be empty"
        assert items, "items must not be empty"

        if batch_id in self._submitted:
            logger.info("batch %s already submitted, skipping", batch_id)
            return BatchResult(
                batch_id=batch_id,
                total=len(items),
                succeeded=0,
                failed=0,
                items=[
                    ItemResult(
                        staff_id=it.staff_id,
                        phone_number=it.phone_number,
                        status="already_submitted",
                    )
                    for it in items
                ],
            )

        request = BatchRequest(batch_id=batch_id, items=items)
        url = f"{self._base_url}{_BATCH_ENDPOINT}"

        async with httpx.AsyncClient() as http:
            resp = await http.post(
                url,
                json=request.model_dump(),
                auth=(self._consumer_key, self._consumer_secret),
                timeout=30.0,
            )
            resp.raise_for_status()
            payload: dict = resp.json()

        self._submitted.add(batch_id)

        raw_items: list[dict] = payload.get("items", [])
        result_items = [
            ItemResult(
                staff_id=r.get("staff_id", ""),
                phone_number=r.get("phone_number", ""),
                status=r.get("status", "unknown"),
                conversation_id=r.get("conversation_id"),
                originator_conversation_id=r.get("originator_conversation_id"),
                error=r.get("error"),
            )
            for r in raw_items
        ]

        succeeded = sum(1 for r in result_items if r.status == "success")
        failed = len(result_items) - succeeded

        return BatchResult(
            batch_id=batch_id,
            total=len(result_items),
            succeeded=succeeded,
            failed=failed,
            items=result_items,
        )
