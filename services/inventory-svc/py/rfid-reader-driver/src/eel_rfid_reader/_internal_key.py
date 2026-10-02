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

"""Internal key derivation helper (not part of public API)."""

from __future__ import annotations

import hashlib


def _derive_key(seed: str, tag_epc: str) -> bytes:
    """Derive a 32-byte HMAC key from (seed, tag_epc) using SHA-256."""
    return hashlib.sha256(f"{seed}:{tag_epc}".encode()).digest()
