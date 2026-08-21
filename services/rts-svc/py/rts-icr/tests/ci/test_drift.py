from rts_icr.drift import DRIFT_THRESHOLD, DriftMonitor


def test_no_drift_at_baseline():
    m = DriftMonitor(baseline_mean=0.92)
    for _ in range(20):
        m.record(0.92)
    assert not m.check_drift()


def test_drift_detected():
    m = DriftMonitor(baseline_mean=0.92)
    for _ in range(20):
        m.record(0.50)
    assert m.check_drift()
