from shangrla.witness import WitnessWorkflow


def test_register_and_list():
    wf = WitnessWorkflow()
    w = wf.register("run-1", "Alice", "IEBC")
    assert w.witness_id
    assert wf.list_witnesses("run-1") == [w]


def test_record_signature():
    wf = WitnessWorkflow()
    w = wf.register("run-2", "Bob", "Org")
    updated = wf.record_signature(w.witness_id, b"sig", "2027-08-09T06:00:00Z")
    assert updated is not None
    assert updated.signature_b64 is not None
