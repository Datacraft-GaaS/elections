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

import numpy as np

_log_ = logging.getLogger(__name__)

DRIFT_THRESHOLD = 0.15


class DriftMonitor:
    """Distribution drift monitor for OCR confidence scores.

    Compares the running mean of observed confidences against a baseline.
    Emits a warning when drift exceeds DRIFT_THRESHOLD.
    """

    def __init__(self, baseline_mean: float = 0.92, baseline_std: float = 0.05) -> None:
        self.baseline_mean = baseline_mean
        self.baseline_std = baseline_std
        self._samples: list[float] = []

    def record(self, confidence: float) -> None:
        self._samples.append(confidence)

    def check_drift(self) -> bool:
        if len(self._samples) < 10:
            return False
        current_mean = float(np.mean(self._samples))
        drift = abs(current_mean - self.baseline_mean)
        if drift > DRIFT_THRESHOLD:
            _log_.warning(
                "drift.detected current_mean=%.3f baseline=%.3f drift=%.3f",
                current_mean,
                self.baseline_mean,
                drift,
            )
            return True
        return False
