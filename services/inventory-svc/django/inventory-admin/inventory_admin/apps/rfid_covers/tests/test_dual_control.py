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

"""Pure-logic tests for the dual-control tamper-resolution rule (HM1)."""

import unittest

from inventory_admin.apps.rfid_covers.dual_control import (
    DualApprovalError,
    countersigned_note,
    validate_dual_approval,
)


class ValidateDualApprovalTests(unittest.TestCase):
    def test_accepts_distinct_second_officer(self):
        self.assertEqual(validate_dual_approval("officer-1", "officer-2"), "officer-2")

    def test_rejects_missing_second_officer(self):
        with self.assertRaises(DualApprovalError):
            validate_dual_approval("officer-1", None)
        with self.assertRaises(DualApprovalError):
            validate_dual_approval("officer-1", "   ")

    def test_rejects_self_approval(self):
        with self.assertRaises(DualApprovalError):
            validate_dual_approval("officer-1", "officer-1")
        # Whitespace-padded self approval also rejected.
        with self.assertRaises(DualApprovalError):
            validate_dual_approval("officer-1", " officer-1 ")

    def test_countersigned_note_carries_both_ids(self):
        note = countersigned_note("alice", "bob", "seal broken in transit")
        self.assertIn("[resolved by alice; countersigned by bob]", note)
        self.assertIn("seal broken in transit", note)

    def test_countersigned_note_defaults_empty_base(self):
        note = countersigned_note("alice", "bob", "")
        self.assertTrue(note.startswith("["))


if __name__ == "__main__":
    unittest.main()
