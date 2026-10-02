// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Pact-style test double for the Gov Printer HTTP endpoint.
//! Returns canned receipt IDs for contract testing.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A simple in-memory double that records submissions and returns canned receipts.
#[derive(Clone, Default)]
pub struct MockGovPrinter {
    submissions: Arc<Mutex<Vec<String>>>,
    next_receipt: Arc<Mutex<String>>,
}

impl MockGovPrinter {
    pub fn new() -> Self {
        Self {
            submissions: Arc::new(Mutex::new(Vec::new())),
            next_receipt: Arc::new(Mutex::new("mock-receipt-001".into())),
        }
    }

    /// Set the receipt ID that the next submission will receive.
    pub fn set_next_receipt(&self, id: impl Into<String>) {
        *self.next_receipt.lock().unwrap() = id.into();
    }

    /// Simulate handling a submission. Returns a receipt body.
    pub fn handle_submission(&self, artifact_id: &str) -> HashMap<String, String> {
        let receipt = self.next_receipt.lock().unwrap().clone();
        self.submissions.lock().unwrap().push(artifact_id.into());
        let mut m = HashMap::new();
        m.insert("submission_id".into(), receipt);
        m.insert("submitted_at".into(), "2026-01-01T00:00:00Z".into());
        m
    }

    /// Return all artifact IDs that have been submitted.
    pub fn submitted_artifact_ids(&self) -> Vec<String> {
        self.submissions.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_returns_canned_receipt() {
        let mock = MockGovPrinter::new();
        let resp = mock.handle_submission("artifact-001");
        assert_eq!(resp["submission_id"], "mock-receipt-001");
    }

    #[test]
    fn mock_records_submissions() {
        let mock = MockGovPrinter::new();
        mock.handle_submission("a1");
        mock.handle_submission("a2");
        let ids = mock.submitted_artifact_ids();
        assert!(ids.contains(&"a1".into()));
        assert!(ids.contains(&"a2".into()));
    }

    #[test]
    fn mock_configurable_receipt() {
        let mock = MockGovPrinter::new();
        mock.set_next_receipt("custom-receipt-999");
        let resp = mock.handle_submission("a1");
        assert_eq!(resp["submission_id"], "custom-receipt-999");
    }
}
