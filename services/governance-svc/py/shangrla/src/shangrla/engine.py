import hashlib
from .models import SampledBallot, TStatResult, AuditRun


class ShangrlaEngine:
    """SHANGRLA ballot-polling and comparison audit engine (WP-5.1 stub)."""

    def compute_t_statistic(
        self, ballots: list[SampledBallot], margin: float = 0.05
    ) -> float:
        """Ballot-polling T-statistic: product of ballot weights."""
        if not ballots:
            return 1.0
        t = 1.0
        for b in ballots:
            w = 1.0 / (2.0 * (1.0 - margin / 2.0)) if not b.discrepancy else 0.0
            t *= w if w > 0 else 0.001
        return t

    def evaluate_run(
        self, run: AuditRun, ballots: list[SampledBallot], margin: float = 0.05
    ) -> TStatResult:
        t = self.compute_t_statistic(ballots, margin)
        risk_achieved = 1.0 / t if t > 0 else 1.0
        passed = risk_achieved <= run.risk_limit
        return TStatResult(
            t_statistic=t,
            risk_achieved=risk_achieved,
            passed=passed,
            recount_triggered=not passed,
        )

    def draw_sample(self, manifest_size: int, seed: str, n: int) -> list[int]:
        """Draw n ballot positions using seed (stub: deterministic from seed hash)."""
        h = int(hashlib.sha256(seed.encode()).hexdigest(), 16)
        return [(h + i * 1337) % max(manifest_size, 1) for i in range(n)]
