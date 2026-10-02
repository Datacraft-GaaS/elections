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

//! Pact-style test double for the KBC SFTP bridge endpoint.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct MockKbcSftp {
    uploads: Arc<Mutex<Vec<String>>>,
    next_remote_path: Arc<Mutex<String>>,
}

impl MockKbcSftp {
    pub fn new() -> Self {
        Self {
            uploads: Arc::new(Mutex::new(Vec::new())),
            next_remote_path: Arc::new(Mutex::new("/kbc/upload/mock.tar.gz".into())),
        }
    }

    pub fn set_next_remote_path(&self, path: impl Into<String>) {
        *self.next_remote_path.lock().unwrap() = path.into();
    }

    pub fn handle_upload(&self, artifact_id: &str, sha256: &str) -> HashMap<String, String> {
        let remote_path = self.next_remote_path.lock().unwrap().clone();
        self.uploads.lock().unwrap().push(artifact_id.into());
        let mut m = HashMap::new();
        m.insert("remote_path".into(), remote_path);
        m.insert("sha256".into(), sha256.into());
        m.insert("uploaded_at".into(), "2026-01-01T00:00:00Z".into());
        m
    }

    pub fn uploaded_artifact_ids(&self) -> Vec<String> {
        self.uploads.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_returns_canned_path() {
        let mock = MockKbcSftp::new();
        let resp = mock.handle_upload("artifact-001", "abc123");
        assert_eq!(resp["remote_path"], "/kbc/upload/mock.tar.gz");
        assert_eq!(resp["sha256"], "abc123");
    }

    #[test]
    fn mock_configurable_path() {
        let mock = MockKbcSftp::new();
        mock.set_next_remote_path("/kbc/custom/path.tar.gz");
        let resp = mock.handle_upload("a", "h");
        assert_eq!(resp["remote_path"], "/kbc/custom/path.tar.gz");
    }

    #[test]
    fn mock_records_uploads() {
        let mock = MockKbcSftp::new();
        mock.handle_upload("a1", "h1");
        mock.handle_upload("a2", "h2");
        let ids = mock.uploaded_artifact_ids();
        assert_eq!(ids.len(), 2);
    }
}
