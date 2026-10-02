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

//! Handoff token redemption.
//! FR-M04-014: one-time QR handoff token lookup.
//! FR-M04-018: constant-time response; timing delta p95 < 5ms between valid/expired/consumed.
//! AC-M04-08: consumed token returns 410 Gone.
//! EC-SEC-10: no timing oracle for token existence.

use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};
use sqlx::Row;
use tracing::info;

use crate::{error::ApiError, state::AppState};

/// GET /vprs/v1/handoff/{token}
///
/// Returns 200 if token is valid, 410 if consumed, 404 if not found/expired.
/// All branches execute equivalent DB work to prevent timing-based token enumeration.
#[allow(dead_code)]
pub async fn get_handoff(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Json<Value>, ApiError> {
    // Hash the token before lookup so raw token never hits the DB query log.
    let token_hash = sha256_hex(token.as_bytes());

    let row = sqlx::query("SELECT app_id, status FROM vprs_applications WHERE qr_token_hash = $1")
        .bind(&token_hash)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| ApiError::ServiceUnavailable)?;

    match row {
        Some(r) => {
            let app_id: String = r.get("app_id");
            let status: String = r.get("status");
            if status == "converted" {
                // Consumed token: 410 Gone per FR-M04-018 / AC-M04-08.
                info!(app_id = %app_id, "handoff token already consumed");
                Err(ApiError::Gone)
            } else if status == "qr_issued" {
                Ok(Json(json!({ "app_id": app_id })))
            } else {
                Err(ApiError::NotFound)
            }
        }
        None => Err(ApiError::NotFound),
    }
}

fn sha256_hex(data: &[u8]) -> String {
    use std::fmt::Write;
    // Minimal SHA-256 via the crypto crate pathway; using a simple std approach here.
    // In production wire through eel-crypto-rs.
    let digest = {
        let mut hasher = Sha256State::new();
        hasher.update(data);
        hasher.finalize()
    };
    let mut hex = String::with_capacity(64);
    for byte in &digest {
        write!(hex, "{byte:02x}").unwrap();
    }
    hex
}

// Minimal SHA-256 implementation to avoid pulling in extra crates beyond what the workspace provides.
struct Sha256State {
    state: [u32; 8],
    buf: Vec<u8>,
}

impl Sha256State {
    #[allow(clippy::unreadable_literal)]
    fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c,
                0x1f83d9ab, 0x5be0cd19,
            ],
            buf: Vec::new(),
        }
    }

    fn update(&mut self, data: &[u8]) {
        self.buf.extend_from_slice(data);
    }

    fn finalize(mut self) -> [u8; 32] {
        let bit_len = (self.buf.len() as u64) * 8;
        self.buf.push(0x80);
        while self.buf.len() % 64 != 56 {
            self.buf.push(0x00);
        }
        self.buf.extend_from_slice(&bit_len.to_be_bytes());

        #[allow(clippy::unreadable_literal)]
        const K: [u32; 64] = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
            0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
            0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
            0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
            0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
            0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
            0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
            0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
            0xc67178f2,
        ];

        for chunk in self.buf.chunks(64) {
            let mut w = [0u32; 64];
            for (i, word) in chunk.chunks(4).enumerate().take(16) {
                w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
            }
            for i in 16..64 {
                let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
                let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
                w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
            }
            let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;
            for i in 0..64 {
                let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                let ch = (e & f) ^ ((!e) & g);
                let temp1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
                let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                let maj = (a & b) ^ (a & c) ^ (b & c);
                let temp2 = s0.wrapping_add(maj);
                h = g; g = f; f = e; e = d.wrapping_add(temp1);
                d = c; c = b; b = a; a = temp1.wrapping_add(temp2);
            }
            self.state[0] = self.state[0].wrapping_add(a);
            self.state[1] = self.state[1].wrapping_add(b);
            self.state[2] = self.state[2].wrapping_add(c);
            self.state[3] = self.state[3].wrapping_add(d);
            self.state[4] = self.state[4].wrapping_add(e);
            self.state[5] = self.state[5].wrapping_add(f);
            self.state[6] = self.state[6].wrapping_add(g);
            self.state[7] = self.state[7].wrapping_add(h);
        }

        let mut out = [0u8; 32];
        for (i, word) in self.state.iter().enumerate() {
            out[i * 4..(i + 1) * 4].copy_from_slice(&word.to_be_bytes());
        }
        out
    }
}
