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

// replay-tally-rs/src/main.rs
//
// Reads a JSON Lines event file, replays events through a BLAKE3-based Merkle
// chain (RFC 8785 canonical JSON per event), and prints a JSON result with the
// final Merkle root.
//
// RFC 8785 canonical JSON (JCS) is implemented inline so this binary has no
// workspace coupling.
//
// Usage: replay-tally-rs <events.jsonl>
//        Prints: {"merkle_root":"<hex>","event_count":<n>}

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;

mod jcs;
mod merkle;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: replay-tally-rs <events.jsonl>");
        process::exit(1);
    }

    let path = &args[1];
    let file = File::open(path).unwrap_or_else(|e| {
        eprintln!("ERROR: cannot open {path}: {e}");
        process::exit(1);
    });

    let result = replay_events(BufReader::new(file));

    println!(
        "{{\"merkle_root\":\"{}\",\"event_count\":{}}}",
        result.root_hex, result.event_count
    );
}

pub struct ReplayResult {
    pub root_hex: String,
    pub event_count: usize,
}

pub fn replay_events<R: BufRead>(reader: R) -> ReplayResult {
    let mut chain = merkle::MerkleChain::new();
    let mut count = 0usize;

    for (line_no, line) in reader.lines().enumerate() {
        let line = line.unwrap_or_else(|e| {
            eprintln!("ERROR: read error at line {}: {e}", line_no + 1);
            process::exit(1);
        });
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(trimmed).unwrap_or_else(|e| {
            eprintln!("ERROR: JSON parse at line {}: {e}", line_no + 1);
            process::exit(1);
        });
        let canonical = jcs::canonicalize(&value).unwrap_or_else(|e| {
            eprintln!("ERROR: JCS canonicalize at line {}: {e}", line_no + 1);
            process::exit(1);
        });
        chain.ingest(canonical.as_bytes());
        count += 1;
    }

    ReplayResult {
        root_hex: chain.finalize(),
        event_count: count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // Expected root for empty stream: the MerkleChain seed is BLAKE3(b"").
    // Actual value verified by running the binary: see merkle::MerkleChain::new().
    const BLAKE3_EMPTY_HEX: &str =
        "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

    #[test]
    fn empty_stream_returns_zero_root() {
        let r = replay_events(Cursor::new(""));
        assert_eq!(r.event_count, 0);
        // Empty chain: finalize on the seed produces the BLAKE3 of empty bytes.
        assert_eq!(r.root_hex, BLAKE3_EMPTY_HEX);
    }

    #[test]
    fn round_trip_small_event_stream() {
        let events = r#"{"seq":1,"type":"eel.tally.vote_cast","count":10}
{"seq":2,"type":"eel.tally.vote_cast","count":8}
{"seq":3,"type":"eel.tally.vote_cast","count":12}"#;

        let r = replay_events(Cursor::new(events));
        assert_eq!(r.event_count, 3);
        // Root is a 64-char hex string.
        assert_eq!(r.root_hex.len(), 64);
        // Same input must always produce the same root (determinism).
        let r2 = replay_events(Cursor::new(events));
        assert_eq!(r.root_hex, r2.root_hex);
    }

    #[test]
    fn tampered_event_detected() {
        let original = r#"{"count":10,"seq":1,"type":"eel.tally.vote_cast"}"#;
        let tampered = r#"{"count":11,"seq":1,"type":"eel.tally.vote_cast"}"#;

        let r_orig = replay_events(Cursor::new(original));
        let r_tamper = replay_events(Cursor::new(tampered));

        assert_ne!(
            r_orig.root_hex, r_tamper.root_hex,
            "tampered event must produce a different root"
        );
    }

    #[test]
    fn canonical_json_is_sorted() {
        // JCS sorts keys; these two must canonicalize to the same bytes,
        // producing the same root.
        let a = r#"{"b":2,"a":1}"#;
        let b = r#"{"a":1,"b":2}"#;
        let ra = replay_events(Cursor::new(a));
        let rb = replay_events(Cursor::new(b));
        assert_eq!(ra.root_hex, rb.root_hex, "JCS must normalize key order");
    }

    #[test]
    fn blank_lines_ignored() {
        let with_blanks = "\n{\"seq\":1}\n\n{\"seq\":2}\n";
        let without = "{\"seq\":1}\n{\"seq\":2}";
        let ra = replay_events(Cursor::new(with_blanks));
        let rb = replay_events(Cursor::new(without));
        assert_eq!(ra.event_count, 2);
        assert_eq!(ra.root_hex, rb.root_hex);
    }
}
