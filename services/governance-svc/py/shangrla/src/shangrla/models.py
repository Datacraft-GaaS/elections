from pydantic import BaseModel, ConfigDict


class AuditMethod(str):
    BALLOT_POLLING = "ballot_polling"
    COMPARISON = "comparison"
    BATCH_COMPARISON = "batch_comparison"


class AuditRun(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )
    run_id: str
    election_id: str
    position_code: str
    audit_method: str
    risk_limit: float
    seed: str | None = None
    status: str = "initialised"


class SampledBallot(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )
    ballot_id: str
    run_id: str
    station_code: str
    batch_id: str
    ballot_position: int
    auditor_interpretation: str | None = None
    discrepancy: bool = False


class TStatResult(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )
    t_statistic: float
    risk_achieved: float
    passed: bool
    recount_triggered: bool


class WitnessRecord(BaseModel):
    model_config = ConfigDict(
        extra="forbid", validate_by_name=True, validate_by_alias=True
    )
    witness_id: str
    run_id: str
    name: str
    organisation: str
    signature_b64: str | None = None
    witnessed_at: str | None = None
