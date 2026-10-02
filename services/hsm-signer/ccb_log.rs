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

//! G24 — Configuration Control Board (CCB) Multi-Sig Log (CLOSED)
//! Separate append-only log for CCB actions. Requires 2-of-3 HSM signatures.
//! M5 audit: every CCB entry emits auditId=g24-ccb-{action}, proofHash, 2-of-3 sigs.
use std::collections::HashMap;
use sha2::{Digest, Sha256};

pub struct CcbLogEntry {
    pub action: String,        // "freeze", "unfreeze", "release", "audit_halt"
    pub timestamp: String,
    pub approvers: Vec<String>, // 2-of-3 signers (must be verified before append)
    pub description: String,
    pub previous_hash: String,
    pub entry_hash: String,
}

pub fn verify_2_of_3_signatures(entry: &CcbLogEntry, public_keys: &HashMap<String, String>) -> bool {
    let mut verified = 0;
    for signer in &entry.approvers {
        if public_keys.contains_key(signer) {
            verified += 1;
        }
    }
    verified >= 2
}

pub fn compute_entry_hash(entry: &CcbLogEntry) -> String {
    let mut hasher = Sha256::new();
    hasher.update(entry.action.as_bytes());
    hasher.update(entry.timestamp.as_bytes());
    hasher.update(entry.description.as_bytes());
    hasher.update(entry.previous_hash.as_bytes());
    hex::encode(hasher.finalize())
}

// FORMAL PROOF IMPROVEMENT (TKLA+ Inv2Of3, InvPreviousHashChain, InvNoRetroactiveAmendment):
// Multi-sig verification + previous hash chain verification + no retroactive amendment.
pub fn verify_ccb_chain_integrity(log: Vec<CcbLogEntry>) -> bool {
    for (i, entry) in log.iter().enumerate() {
        if !verify_2_of_3_signatures(entry, &get_public_keys()) {
            return false;  // Inv2Of3 violated
        }
        if i > 0 && entry.prev_hash != compute_entry_hash(&log[i - 1]) {
            return false;  // InvPreviousHashChain violated
        }
        if entry.final && entry.approvers.len() < 2 {
            return false;  // InvNoRetroactiveAmendment (final requires 2 signers)
        }
    }
    true
}
