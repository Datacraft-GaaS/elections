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

"""eel-bagit: BagIt 1.0 package builder and verifier for WP-5.6 M69."""

from .builder import BagBuilder
from .models import BagInfo, BagManifest, VerificationResult
from .verifier import BagVerifier

__all__ = [
    "BagBuilder",
    "BagVerifier",
    "BagInfo",
    "BagManifest",
    "VerificationResult",
]
