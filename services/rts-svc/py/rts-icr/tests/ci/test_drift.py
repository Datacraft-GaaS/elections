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

from rts_icr.drift import DriftMonitor


def test_no_drift_at_baseline():
    m = DriftMonitor(baseline_mean=0.92)
    for _ in range(20):
        m.record(0.92)
    assert not m.check_drift()


def test_drift_detected():
    m = DriftMonitor(baseline_mean=0.92)
    for _ in range(20):
        m.record(0.50)
    assert m.check_drift()
