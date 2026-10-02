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

from pydantic import BaseModel, ConfigDict


class ImageRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")
    image_b64: str  # base64-encoded PNG/JPEG bytes
    width: int
    height: int


class Circle(BaseModel):
    model_config = ConfigDict(extra="forbid")
    cx: float
    cy: float
    radius: float


class SegmentResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    pupil: Circle
    limbus: Circle
    iris_sclera_ratio: float
    mask_b64: str  # base64-encoded binary mask (width*height bytes)


class EncodeResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    iris_code_b64: str  # base64-encoded 256-byte IrisCode
    mask_b64: str  # base64-encoded 256-byte mask
    quality: float


class QualityResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    sharpness: float
    contrast: float
    dilation: float
    gaze_angle_deg: float
    occlusion_fraction: float
    overall: float


class LensResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    has_lens: bool
    kind: str  # "none" | "transparent" | "coloured" | "patterned"
    confidence: float


class EmbedResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    embedding: list[float]  # 256 floats, L2-normalised


class HealthResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    status: str
    models_loaded: dict[str, bool]
