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

from shangrla.witness import WitnessWorkflow


def test_register_and_list():
    wf = WitnessWorkflow()
    w = wf.register("run-1", "Alice", "Electoral Authority")
    assert w.witness_id
    assert wf.list_witnesses("run-1") == [w]


def test_record_signature():
    wf = WitnessWorkflow()
    w = wf.register("run-2", "Bob", "Org")
    updated = wf.record_signature(w.witness_id, b"sig", "2027-08-09T06:00:00Z")
    assert updated is not None
    assert updated.signature_b64 is not None
