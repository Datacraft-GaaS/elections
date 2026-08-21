"""Async IFMIS adapter client (WP-4.12 IFMIS adapter at http://ifmis-adapter:8000)."""

import httpx

from .models import GlPostingRequest, GlPostingResponse, IfmisVoucher


class IfmisClient:
    """Thin async wrapper around the WP-4.12 IFMIS HTTP adapter."""

    def __init__(
        self,
        base_url: str = "http://ifmis-adapter:8000",
        timeout: float = 30.0,
        *,
        client: httpx.AsyncClient | None = None,
    ) -> None:
        self._base_url = base_url.rstrip("/")
        self._timeout = timeout
        self._client = client

    async def _get_client(self) -> httpx.AsyncClient:
        if self._client is not None:
            return self._client
        return httpx.AsyncClient(base_url=self._base_url, timeout=self._timeout)

    async def post_gl(self, req: GlPostingRequest) -> GlPostingResponse:
        """Submit a GL posting to IFMIS and return the posting confirmation."""
        client = await self._get_client()
        resp = await client.post(
            "/api/v1/gl/postings",
            json=req.model_dump(mode="json"),
        )
        resp.raise_for_status()
        return GlPostingResponse.model_validate(resp.json())

    async def get_voucher(self, ifmis_ref: str) -> IfmisVoucher:
        """Retrieve a voucher by its IFMIS reference number."""
        client = await self._get_client()
        resp = await client.get(f"/api/v1/vouchers/{ifmis_ref}")
        resp.raise_for_status()
        return IfmisVoucher.model_validate(resp.json())

    async def list_vouchers(
        self,
        budget_line_code: str | None = None,
        status: str | None = None,
    ) -> list[IfmisVoucher]:
        """List vouchers, optionally filtered by budget line code or status."""
        client = await self._get_client()
        params: dict[str, str] = {}
        if budget_line_code:
            params["budget_line_code"] = budget_line_code
        if status:
            params["status"] = status
        resp = await client.get("/api/v1/vouchers", params=params)
        resp.raise_for_status()
        data = resp.json()
        return [IfmisVoucher.model_validate(item) for item in data.get("items", [])]
