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
from forensics_ml.engine import DiDEngine, IvEngine, RdEngine
from forensics_ml.models import RdInput
from forensics_ml.service import ForensicsMLService


def test_rd_returns_estimate():
    engine = RdEngine()
    # 20 observations on each side of threshold 0.5
    vote_share = [0.4 + 0.01 * i for i in range(20)] + [
        0.5 + 0.01 * i for i in range(20)
    ]
    expenditure = [0.4 + 0.01 * i for i in range(20)] + [
        0.5 + 0.01 * i for i in range(20)
    ]
    est, ci_low, ci_high, pv = engine.estimate(
        vote_share, expenditure, threshold=0.5, bandwidth=0.15
    )
    assert isinstance(est, float)
    assert ci_low <= ci_high
    assert 0.0 <= pv <= 1.0


def test_iv_returns_estimate():
    engine = IvEngine()
    import numpy as np

    rng = np.random.default_rng(42)
    n = 100
    z = rng.normal(size=n).tolist()
    d = (rng.normal(size=n) + z).tolist()
    y = (rng.normal(size=n) + d).tolist()
    est, ci_low, ci_high, pv = engine.estimate(y, d, z)
    assert isinstance(est, float)
    assert ci_low <= ci_high


def test_did_att_sign():
    engine = DiDEngine()
    # Treated group improves more than control
    pre = [1.0] * 10 + [1.0] * 10
    post = [2.0] * 10 + [1.2] * 10  # treated gain 1.0, control gain 0.2
    treated = [True] * 10 + [False] * 10
    att, _, _, _ = engine.estimate(pre, post, treated)
    assert att > 0  # positive ATT


async def test_service_run_rd():
    svc = ForensicsMLService()
    inp = RdInput(
        constituency_id="CONST-001",
        vote_share=[0.4 + 0.01 * i for i in range(40)],
        expenditure=[0.3 + 0.01 * i for i in range(40)],
        threshold=0.5,
    )
    result = await svc.run_rd(inp)
    assert result.constituency_id == "CONST-001"
    assert result.method.value == "RD"


async def test_replication_manifest_hashes():
    svc = ForensicsMLService()
    from forensics_ml.models import AnalysisMethod

    manifest = svc.build_replication_manifest(
        "cyc-001", AnalysisMethod.RD, {"data": [1, 2, 3]}, {"result": 0.5}
    )
    assert len(manifest.input_hash) == 64
    assert len(manifest.output_hash) == 64
    assert manifest.cycle_id == "cyc-001"
