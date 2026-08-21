import logging
from .engine import ShangrlaEngine
from .witness import WitnessWorkflow
from .nist_beacon import fetch_latest_pulse, get_beacon_seed
from .iab_seed import derive_iab_card_seed
from .models import AuditRun, SampledBallot, TStatResult, WitnessRecord

_logger = logging.getLogger(__name__)


class GovernanceAuditService:
    """Thin async facade over SHANGRLA engine, witness workflow, and NIST beacon."""

    def __init__(self) -> None:
        self._engine = ShangrlaEngine()
        self._witness = WitnessWorkflow()

    def _log_event(self, event: str, **kwargs: object) -> None:
        _logger.info("governance_audit.%s %s", event, kwargs)

    async def fetch_seed_from_beacon(self, card_serial: str, election_id: str) -> str:
        """Fetch NIST beacon pulse and derive IAB card audit seed."""
        self._log_event("fetch_seed", card_serial=card_serial, election_id=election_id)
        pulse = await fetch_latest_pulse()
        nist_value = await get_beacon_seed(pulse)
        seed = derive_iab_card_seed(card_serial, election_id, nist_value)
        self._log_event("seed_derived", election_id=election_id)
        return seed

    def evaluate(
        self, run: AuditRun, ballots: list[SampledBallot], margin: float = 0.05
    ) -> TStatResult:
        """Evaluate audit run against sampled ballots."""
        self._log_event("evaluate", run_id=run.run_id, ballot_count=len(ballots))
        result = self._engine.evaluate_run(run, ballots, margin)
        self._log_event("evaluate_done", run_id=run.run_id, passed=result.passed)
        return result

    def draw_sample(self, manifest_size: int, seed: str, n: int) -> list[int]:
        """Draw n ballot positions deterministically from seed."""
        self._log_event("draw_sample", manifest_size=manifest_size, n=n)
        return self._engine.draw_sample(manifest_size, seed, n)

    def register_witness(
        self, run_id: str, name: str, organisation: str
    ) -> WitnessRecord:
        self._log_event("register_witness", run_id=run_id, name=name)
        return self._witness.register(run_id, name, organisation)

    def record_witness_signature(
        self, witness_id: str, signature_bytes: bytes, witnessed_at: str
    ) -> WitnessRecord | None:
        self._log_event("record_signature", witness_id=witness_id)
        return self._witness.record_signature(witness_id, signature_bytes, witnessed_at)

    def list_witnesses(self, run_id: str) -> list[WitnessRecord]:
        return self._witness.list_witnesses(run_id)
