import base64
from uuid6 import uuid7
from .models import WitnessRecord


class WitnessWorkflow:
    def __init__(self) -> None:
        self._witnesses: list[WitnessRecord] = []

    def register(self, run_id: str, name: str, organisation: str) -> WitnessRecord:
        w = WitnessRecord(
            witness_id=str(uuid7()),
            run_id=run_id,
            name=name,
            organisation=organisation,
        )
        self._witnesses.append(w)
        return w

    def record_signature(
        self, witness_id: str, signature_bytes: bytes, witnessed_at: str
    ) -> WitnessRecord | None:
        for w in self._witnesses:
            if w.witness_id == witness_id:
                return w.model_copy(
                    update={
                        "signature_b64": base64.b64encode(signature_bytes).decode(),
                        "witnessed_at": witnessed_at,
                    }
                )
        return None

    def list_witnesses(self, run_id: str) -> list[WitnessRecord]:
        return [w for w in self._witnesses if w.run_id == run_id]
