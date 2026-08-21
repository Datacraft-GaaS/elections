"""PHI (Protected Health Information) schema enforcement.

Any incoming data dict must not contain keys that suggest patient identity,
diagnosis, or health records. This enforces EC-DAT-01 (no PHI persistence).
"""

_PHI_KEYWORDS: list[str] = [
    "patient",
    "name",
    "diagnosis",
    "dob",
    "nid",
    "ssn",
    "health_id",
]


def validate_no_phi(data: dict) -> None:
    """Raise ValueError if any key in data matches a PHI keyword (case-insensitive).

    Checks substrings: 'patient_name' triggers on 'patient' and on 'name'.
    """
    for key in data:
        key_lower = key.lower()
        for phi_term in _PHI_KEYWORDS:
            if phi_term in key_lower:
                raise ValueError(
                    f"Field '{key}' is not permitted: contains PHI keyword '{phi_term}'. "
                    "MedicalIncident stores aggregate counts only, no patient data."
                )
