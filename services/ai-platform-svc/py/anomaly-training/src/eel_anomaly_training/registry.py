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

from __future__ import annotations

import hashlib
import json
import shutil
from pathlib import Path

from pydantic import BaseModel, ConfigDict


class ModelCard(BaseModel):
    model_config = ConfigDict(
        extra="forbid",
        validate_by_name=True,
        validate_by_alias=True,
    )

    model_id: str
    model_type: str
    training_date: str
    accuracy: float | None = None
    dataset_hash: str
    algorithm_citation: str
    onnx_path: str | None = None


def write_model_card(card: ModelCard, output_dir: Path) -> Path:
    """Write model card JSON to output_dir/{model_id}.json."""
    output_dir.mkdir(parents=True, exist_ok=True)
    dest = output_dir / f"{card.model_id}.json"
    dest.write_text(card.model_dump_json(indent=2))
    return dest


def cosign_stub(model_path: Path) -> str:
    """Stub cosign signature. Returns cosign://stub/<sha256> of the file."""
    sha = hashlib.sha256(model_path.read_bytes()).hexdigest()
    return f"cosign://stub/{sha}"


def publish_model_card(card: ModelCard, reports_dir: Path) -> Path:
    """Copy model card JSON to reports_dir/{model_id}.json."""
    reports_dir.mkdir(parents=True, exist_ok=True)
    dest = reports_dir / f"{card.model_id}.json"
    dest.write_text(card.model_dump_json(indent=2))
    return dest
