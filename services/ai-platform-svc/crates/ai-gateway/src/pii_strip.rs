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

use sha2::{Digest, Sha256};

fn has_long_digit_run(s: &str) -> bool {
    let mut run = 0usize;
    for c in s.chars() {
        if c.is_ascii_digit() {
            run += 1;
            if run >= 7 {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

/// Strip PII tokens from input text.
///
/// Replaces email-like tokens (containing '@') with [EMAIL_REDACTED]
/// and digit runs of 7+ consecutive digits with [ID_REDACTED].
/// Returns the cleaned text and the count of replacements made.
pub fn strip_pii(input: &str) -> (String, u32) {
    let mut count = 0u32;
    let parts: Vec<&str> = input.split_whitespace().collect();
    let replaced: Vec<String> = parts
        .into_iter()
        .map(|word| {
            if word.contains('@') {
                count += 1;
                "[EMAIL_REDACTED]".to_string()
            } else if has_long_digit_run(word) {
                count += 1;
                "[ID_REDACTED]".to_string()
            } else {
                word.to_string()
            }
        })
        .collect();
    (replaced.join(" "), count)
}

/// SHA-256 hash of the given string, hex-encoded.
pub fn input_hash(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let digest = hasher.finalize();
    digest.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_email() {
        let (out, count) = strip_pii("contact user@example.com for help");
        assert_eq!(count, 1);
        assert!(out.contains("[EMAIL_REDACTED]"));
        assert!(!out.contains("user@example.com"));
    }

    #[test]
    fn strips_long_digit_run() {
        let (out, count) = strip_pii("voter ID is 12345678 in register");
        assert_eq!(count, 1);
        assert!(out.contains("[ID_REDACTED]"));
    }

    #[test]
    fn short_digits_unchanged() {
        let (out, count) = strip_pii("score 123456 on test");
        assert_eq!(count, 0);
        assert!(out.contains("123456"));
    }

    #[test]
    fn input_hash_is_hex() {
        let h = input_hash("hello");
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
