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

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AttestError {
    #[error("attestation quote invalid: {0}")]
    InvalidQuote(String),
    #[error("attestation replay detected")]
    Replay,
    #[error("attestation profile not supported: {0}")]
    UnsupportedProfile(String),
    #[error("enclave quarantined")]
    Quarantined,
    #[error("nonce expired or invalid")]
    NonceExpired,
    #[error("nonce not found")]
    NonceNotFound,
    #[error("policy violation: {0}")]
    PolicyViolation(String),
    #[error("certificate chain verification failed: {0}")]
    ChainVerificationFailed(String),
    #[error("report too short: expected {expected} bytes, got {got}")]
    ReportTooShort { expected: usize, got: usize },
    #[error("software attestation rejected in production policy")]
    SoftwareAttestationRejected,
}
