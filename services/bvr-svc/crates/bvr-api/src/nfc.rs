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

//! Unicode NFC normalization for BVR name and ID fields.
//!
//! Canonicalizes decomposed input (e.g. `é`) to composed form (`é`) so
//! that register matching and deduplication are insensitive to how a source
//! system encoded the same grapheme.

use unicode_normalization::UnicodeNormalization;

/// Normalize a string to Unicode NFC form.
///
/// NFC is idempotent: `nfc(nfc(s)) == nfc(s)`. ASCII input is unchanged.
pub fn nfc(s: impl Into<String>) -> String {
    s.into().nfc().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_trace_rs::trace;

    #[trace("EC-DAT-09")]
    #[test]
    fn nfc_composes_decomposed_accent() {
        // "e" + combining acute accent (U+0065 U+0301) -> "é" (U+00E9).
        assert_eq!(nfc("e\u{0301}"), "\u{00E9}");
    }

    #[trace("EC-DAT-09")]
    #[test]
    fn nfc_is_idempotent() {
        let input = "cafe\u{0301}\u{0327}";
        let once = nfc(input);
        assert_eq!(nfc(once.clone()), once);
    }

    #[trace("EC-DAT-09")]
    #[test]
    fn nfc_passthrough_ascii() {
        assert_eq!(nfc("hello"), "hello");
    }
}
