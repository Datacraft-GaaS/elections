import httpx
from typing import Any

NIST_BEACON_URL = "https://beacon.nist.gov/beacon/2.0/pulse/last"


async def fetch_latest_pulse() -> dict[str, Any]:
    """Fetch latest NIST randomness beacon pulse."""
    async with httpx.AsyncClient(timeout=10.0) as client:
        resp = await client.get(NIST_BEACON_URL)
        resp.raise_for_status()
        return resp.json()


async def get_beacon_seed(pulse: dict[str, Any]) -> str:
    """Extract output value from NIST beacon pulse."""
    return pulse.get("pulse", {}).get("outputValue", "")
