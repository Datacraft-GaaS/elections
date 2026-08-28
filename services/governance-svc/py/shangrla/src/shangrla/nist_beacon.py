import httpx
from typing import Any

# Canonical beacon/seed-mixing implementation lives in beacon.py; re-exported
# here so existing importers of nist_beacon get the verified API too.
from .beacon import (  # noqa: F401
    BeaconClient,
    BeaconPulse,
    combine_seeds,
    commit_seed,
    verify_commitment,
)

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
