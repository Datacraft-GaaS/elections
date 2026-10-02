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

from shangrla.engine import PairwiseAssertion, ShangrlaEngine
from shangrla.models import AuditRun, SampledBallot


def test_t_statistic_empty():
    engine = ShangrlaEngine()
    assert engine.compute_t_statistic([]) == 1.0


def test_evaluate_pass():
    engine = ShangrlaEngine()
    run = AuditRun(
        run_id="r1",
        election_id="e1",
        position_code="P001",
        audit_method="ballot_polling",
        risk_limit=0.05,
    )
    ballots = [
        SampledBallot(
            ballot_id=str(i),
            run_id="r1",
            station_code="S1",
            batch_id="B1",
            ballot_position=i,
        )
        for i in range(5)
    ]
    result = engine.evaluate_run(
        run,
        ballots,
        PairwiseAssertion(winner="A", loser="B", reported_share=0.7),
    )
    assert isinstance(result.passed, bool)
    assert result.t_statistic > 0


def test_draw_sample_deterministic():
    engine = ShangrlaEngine()
    s1 = engine.draw_sample(1000, "seed-abc", 10)
    s2 = engine.draw_sample(1000, "seed-abc", 10)
    assert s1 == s2
    assert len(s1) == 10
