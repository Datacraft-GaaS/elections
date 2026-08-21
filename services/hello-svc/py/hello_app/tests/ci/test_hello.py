import asyncio

from httpx import ASGITransport, AsyncClient

from hello_app.service import app


async def test_root_returns_ok():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        response = await client.get("/")
    assert response.status_code == 200
    assert response.json() == {"ok": True}


def test_root_sync():
    loop = asyncio.get_event_loop()
    loop.run_until_complete(test_root_returns_ok())
