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

"""BagIt bag verifier (BagIt 1.0 spec, RFC 8493)."""

from __future__ import annotations

import hashlib
from pathlib import Path

from .models import VerificationResult


def _sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


class BagVerifier:
    """Verify a BagIt 1.0 bag by checking manifest-sha256.txt against actual file hashes.

    Annual integrity check pattern: call verify() and inspect VerificationResult.valid.
    """

    def verify(self, bag_path: str | Path) -> VerificationResult:
        bag = Path(bag_path)
        errors: list[str] = []

        if not bag.exists():
            return VerificationResult(
                valid=False,
                bag_path=str(bag),
                errors=[f"bag directory does not exist: {bag}"],
            )

        bagit_txt = bag / "bagit.txt"
        if not bagit_txt.exists():
            errors.append("bagit.txt is missing")

        manifest_path = bag / "manifest-sha256.txt"
        if not manifest_path.exists():
            errors.append("manifest-sha256.txt is missing")
            return VerificationResult(valid=False, bag_path=str(bag), errors=errors)

        # Parse manifest: each line is "<sha256hex>  <relative_path>"
        declared: dict[str, str] = {}
        for line_no, raw in enumerate(
            manifest_path.read_text(encoding="utf-8").splitlines(), 1
        ):
            line = raw.strip()
            if not line:
                continue
            parts = line.split("  ", 1)
            if len(parts) != 2:
                errors.append(f"manifest line {line_no} is malformed: {raw!r}")
                continue
            digest, rel_path = parts
            declared[rel_path] = digest

        # Verify each declared file
        for rel_path, expected_digest in declared.items():
            file_path = bag / rel_path
            if not file_path.exists():
                errors.append(f"file missing: {rel_path}")
                continue
            actual = _sha256_file(file_path)
            if actual != expected_digest:
                errors.append(
                    f"checksum mismatch for {rel_path}: expected {expected_digest}, got {actual}"
                )

        # Check for files in data/ not listed in the manifest
        data_dir = bag / "data"
        if data_dir.exists():
            for candidate in sorted(data_dir.rglob("*")):
                if candidate.is_file():
                    rel = str(candidate.relative_to(bag))
                    if rel not in declared:
                        errors.append(f"file in data/ not listed in manifest: {rel}")

        return VerificationResult(
            valid=len(errors) == 0, bag_path=str(bag), errors=errors
        )
