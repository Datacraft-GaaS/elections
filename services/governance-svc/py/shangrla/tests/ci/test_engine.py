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
