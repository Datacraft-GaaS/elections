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

//! Fuzz target: verify that Debug and serde on PII types never expose the
//! raw value, regardless of what string is stored inside.

#![no_main]
use eel_pii_rs::{NoopAuditSink, PIIRestricted, PIISensitive, Purpose};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let r = PIIRestricted::new(s.to_owned());
        let dbg = format!("{r:?}");
        assert!(
            !dbg.contains(s) || s.is_empty() || s.contains("REDACTED"),
            "PIIRestricted Debug leaked value"
        );
        assert!(
            serde_json::to_string(&r).is_err(),
            "PIIRestricted must not serialize"
        );

        let sens = PIISensitive::new(s.to_owned());
        let dbg2 = format!("{sens:?}");
        assert!(
            !dbg2.contains(s) || s.is_empty() || s.contains("REDACTED"),
            "PIISensitive Debug leaked value"
        );
        assert!(
            serde_json::to_string(&sens).is_err(),
            "PIISensitive must not serialize"
        );

        // reveal must return the correct value
        let val = r.reveal(Purpose::Audit, &NoopAuditSink);
        assert_eq!(val, s);
    }
});
