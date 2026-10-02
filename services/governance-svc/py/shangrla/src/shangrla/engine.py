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

"""SHANGRLA/BRAVO ballot-polling audit engine (WP-5.1).

Python twin of the canonical Rust implementation in
`services/governance-svc/crates/rla/src/engine.rs`. Implements BRAVO
(Lindeman, Stark & Yates, EVT/WOTE 2012) via pairwise winner/loser
assertions against the exact-tie null, and deterministic without-
replacement sampling using SHA-256 in counter mode (NIST beacon style).
"""

import hashlib

from .models import AuditRun, SampledBallot, TStatResult

_NULL_SHARE = 0.5


class PairwiseAssertion:
    """Assertion that `winner` truly received more votes than `loser`."""

    def __init__(self, winner: str, loser: str, reported_share: float):
        if not 0.5 < reported_share <= 1.0:
            raise ValueError("reported_share must lie in (0.5, 1.0]")
        self.winner = winner
        self.loser = loser
        self.reported_share = reported_share

    def update_t(self, t: float, audited_vote: str | None) -> float:
        """BRAVO multiplicative update for one audited ballot."""
        if audited_vote == self.winner:
            return t * (self.reported_share / _NULL_SHARE)
        if audited_vote == self.loser:
            return t * ((1.0 - self.reported_share) / _NULL_SHARE)
        return t

    def stop_threshold(self, risk_limit: float) -> float:
        """T must reach 1/alpha to confirm at the target risk limit."""
        alpha = min(max(risk_limit, 1e-12), 1.0)
        return 1.0 / alpha


def _counter_digest(seed_digest: bytes, counter: int) -> bytes:
    h = hashlib.sha256()
    h.update(seed_digest)
    h.update(counter.to_bytes(8, "big"))
    return h.digest()


class ShangrlaEngine:
    """BRAVO ballot-polling audit engine.

    `compute_t_statistic` interprets each ballot's `auditor_interpretation`
    as the audited vote (None/blank = not yet audited and uninformative);
    `discrepancy=True` ballots are treated as votes for the loser (they
    evidence against the reported outcome).
    """

    def compute_t_statistic(
        self,
        ballots: list[SampledBallot],
        assertion: PairwiseAssertion | None = None,
    ) -> float:
        t = 1.0
        for b in ballots:
            vote = b.auditor_interpretation or None
            if b.discrepancy:
                # A discrepancy is evidence the recorded outcome is wrong.
                vote = assertion.loser if assertion else vote
            if assertion is not None:
                t = assertion.update_t(t, vote)
        return max(t, 1e-12)

    def evaluate_run(
        self,
        run: AuditRun,
        ballots: list[SampledBallot],
        assertion: PairwiseAssertion,
    ) -> TStatResult:
        t = self.compute_t_statistic(ballots, assertion)
        threshold = assertion.stop_threshold(run.risk_limit)
        passed = t >= threshold
        risk_achieved = 1.0 / t
        return TStatResult(
            t_statistic=t,
            risk_achieved=risk_achieved,
            passed=passed,
            recount_triggered=not passed,
        )

    def draw_sample(self, manifest_size: int, seed: str, n: int) -> list[int]:
        """Draw n ballot indices without replacement, deterministically.

        SHA-256 counter-mode over the seed; duplicate hits are rejected so
        the sample is a true simple random sample without replacement.
        """
        if n > manifest_size:
            raise ValueError("sample larger than manifest")
        seed_digest = hashlib.sha256(seed.encode()).digest()
        seen: set[int] = set()
        out: list[int] = []
        counter = 0
        while len(out) < n:
            digest = _counter_digest(seed_digest, counter)
            counter += 1
            idx = int.from_bytes(digest[:8], "big") % max(manifest_size, 1)
            if idx in seen:
                continue
            seen.add(idx)
            out.append(idx)
        return sorted(out)


__all__ = ["PairwiseAssertion", "ShangrlaEngine"]
