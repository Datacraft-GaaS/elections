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

import logging

from .models import IcrField

_log_ = logging.getLogger(__name__)


class OcrUnavailableError(RuntimeError):
    """Raised when no ICR handwriting model is available for extraction."""


async def extract_fields(image_path: str, form_type: str = "34a") -> list[IcrField]:
    """Extract form fields from a scanned Form 34A image.

    Extraction requires the PaddleOCR ONNX handwriting-recognition model,
    which is not yet vendored (see `docs/plans/deferral-plan.md` section D).
    Until that model lands, field extraction fails loudly instead of
    fabricating synthetic results.
    """
    _log_.warning(
        "ocr.extract_fields unavailable form_type=%s path=%s (see deferral-plan.md section D)",
        form_type,
        image_path,
    )
    raise OcrUnavailableError(
        "ICR handwriting model not vendored; see docs/plans/deferral-plan.md"
    )
