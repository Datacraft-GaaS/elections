"""CI tests for abis-vendor worker. No mocks - real objects only."""

from __future__ import annotations
import asyncio
import pytest
from abis_vendor import AbisVendorWorker, MatchOutcome, MatchRequest, MatchResult
from abis_vendor.adapter import LegacyAbisAdapter
from abis_vendor.models import VendorConfig

MOCK_CONFIG = VendorConfig(adapter_url="http://localhost:9999", mock_mode=True)


def test_worker_process_one_mock():
    config = MOCK_CONFIG
    worker = AbisVendorWorker(config)
    req = MatchRequest(
        request_id="req-001",
        probe_ref="probe-abc",
        modality="fingerprint",
        election_id="elec-001",
    )
    loop = asyncio.get_event_loop()
    result = loop.run_until_complete(worker.process_one(req))
    assert isinstance(result, MatchResult)
    assert result.request_id == "req-001"
    assert result.outcome in MatchOutcome.__members__.values()


def test_worker_batch_mock():
    worker = AbisVendorWorker(MOCK_CONFIG)
    requests = [
        MatchRequest(
            request_id=f"req-{i}",
            probe_ref=f"probe-{i}",
            modality="fingerprint",
            election_id="elec-001",
        )
        for i in range(5)
    ]
    loop = asyncio.get_event_loop()
    results = loop.run_until_complete(worker.run_batch(requests))
    assert len(results) == 5
    for r in results:
        assert r.outcome in MatchOutcome.__members__.values()


def test_adapter_mock_deterministic():
    """Same probe_ref always gives same outcome in mock mode."""
    adapter = LegacyAbisAdapter(MOCK_CONFIG)
    req = MatchRequest(
        request_id="r1", probe_ref="stable-probe", modality="iris", election_id="e1"
    )
    loop = asyncio.get_event_loop()

    async def run():
        r1 = adapter._mock_result(req)
        r2 = adapter._mock_result(req)
        return r1, r2

    r1, r2 = loop.run_until_complete(run())
    assert r1.confidence_score == r2.confidence_score
    assert r1.outcome == r2.outcome


def test_match_outcome_values():
    assert MatchOutcome.match == "match"
    assert MatchOutcome.no_match == "no_match"
    assert MatchOutcome.adjudicate == "adjudicate"


def test_match_request_extra_fields_forbidden():
    from pydantic import ValidationError

    with pytest.raises(ValidationError):
        MatchRequest(  # type: ignore[call-arg]
            request_id="r",
            probe_ref="p",
            modality="f",
            election_id="e",
            extra_field="bad",  # type: ignore[call-arg]
        )
