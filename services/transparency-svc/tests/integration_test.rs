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

//! Integration tests for the transparency log service.

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use tlog_domain::{Domain, LeafFormat};

    #[test]
    fn leaf_hash_smoke() {
        let t = Utc::now();
        let h = LeafFormat::hash(b"test event", "spiffe://eel.local/test", t, 0);
        assert_eq!(h.len(), 32);
    }

    #[test]
    fn domain_all_variants_parse() {
        use std::str::FromStr;
        for d in [
            "voter-events",
            "results-events",
            "custody-events",
            "credential-events",
            "meta-events",
        ] {
            assert!(Domain::from_str(d).is_ok(), "failed to parse domain: {}", d);
        }
    }
}
