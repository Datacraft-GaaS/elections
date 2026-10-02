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

import hashlib


def derive_iab_card_seed(card_serial: str, election_id: str, nist_value: str) -> str:
    """Derive audit seed from IAB card serial, election ID, and NIST beacon value."""
    combined = f"{card_serial}:{election_id}:{nist_value}"
    return hashlib.sha256(combined.encode()).hexdigest()
