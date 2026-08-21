import hashlib
from datetime import datetime, timezone

from uuid6 import uuid7

from .models import uuid7str


def generate_certificate(worker_id: str, station_code: str, modules: list[str]) -> dict:
    issued_at = datetime.now(tz=timezone.utc).isoformat()
    raw = f"{worker_id}:{station_code}:{issued_at}"
    cert_hash = hashlib.sha256(raw.encode()).hexdigest()
    cert_id = uuid7str()
    qr_url = f"https://verify.iebc.or.ke/cert/{cert_id}"
    return {
        "id": cert_id,
        "hash": cert_hash,
        "qr_url": qr_url,
        "issued_at": issued_at,
        "worker_id": worker_id,
        "station_code": station_code,
        "modules_completed": modules,
    }
