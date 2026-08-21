from __future__ import annotations
import hashlib
import json
import logging
from typing import Any
from datetime import UTC, datetime

from .engine import DiDEngine, IvEngine, RdEngine
from .models import (
    AnalysisMethod,
    DiDInput,
    EstimateResult,
    IvInput,
    RdInput,
    ReplicationManifest,
)

_logger = logging.getLogger(__name__)


class ForensicsMLService:
    """Orchestrates RD/IV/DiD analysis and replication package generation."""

    def __init__(self) -> None:
        self._rd = RdEngine()
        self._iv = IvEngine()
        self._did = DiDEngine()

    def _log_event(self, event: str, **kw: Any) -> None:
        _logger.info("forensics_ml.%s %s", event, kw)

    async def run_rd(self, inp: RdInput) -> EstimateResult:
        self._log_event("run_rd", constituency_id=inp.constituency_id)
        est, ci_low, ci_high, pv = self._rd.estimate(
            inp.vote_share, inp.expenditure, inp.threshold, inp.bandwidth
        )
        return EstimateResult(
            constituency_id=inp.constituency_id,
            method=AnalysisMethod.RD,
            estimate=est,
            ci_low=ci_low,
            ci_high=ci_high,
            p_value=pv,
        )

    async def run_iv(self, inp: IvInput) -> EstimateResult:
        self._log_event("run_iv", constituency_id=inp.constituency_id)
        est, ci_low, ci_high, pv = self._iv.estimate(
            inp.outcome, inp.endogenous, inp.instrument
        )
        return EstimateResult(
            constituency_id=inp.constituency_id,
            method=AnalysisMethod.IV,
            estimate=est,
            ci_low=ci_low,
            ci_high=ci_high,
            p_value=pv,
        )

    async def run_did(self, inp: DiDInput) -> EstimateResult:
        self._log_event("run_did", constituency_id=inp.constituency_id)
        est, ci_low, ci_high, pv = self._did.estimate(
            inp.outcome_pre, inp.outcome_post, inp.treated
        )
        return EstimateResult(
            constituency_id=inp.constituency_id,
            method=AnalysisMethod.DID,
            estimate=est,
            ci_low=ci_low,
            ci_high=ci_high,
            p_value=pv,
        )

    def build_replication_manifest(
        self,
        cycle_id: str,
        method: AnalysisMethod,
        input_data: dict[str, Any],
        output_data: dict[str, Any],
    ) -> ReplicationManifest:
        """Build replication manifest for M14 reproducibility bundle."""
        self._log_event("build_replication_manifest", cycle_id=cycle_id, method=method)
        input_hash = hashlib.sha256(
            json.dumps(input_data, sort_keys=True).encode()
        ).hexdigest()
        output_hash = hashlib.sha256(
            json.dumps(output_data, sort_keys=True).encode()
        ).hexdigest()
        return ReplicationManifest(
            cycle_id=cycle_id,
            method=method,
            input_hash=input_hash,
            output_hash=output_hash,
            environment_spec="python>=3.12,numpy>=1.26,scipy>=1.13,statsmodels>=0.14",
            produced_at=datetime.now(UTC).isoformat(),
        )
