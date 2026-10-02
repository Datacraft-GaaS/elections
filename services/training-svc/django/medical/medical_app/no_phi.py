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
