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
pub enum IcaoError {
    #[error("invalid TLV tag: 0x{0:02X}")]
    InvalidTag(u8),

    #[error("invalid data length")]
    InvalidLength,

    #[error("invalid MRZ: {0}")]
    InvalidMrz(String),

    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("signature verification failed")]
    SignatureVerificationFailed,

    #[error("key derivation failed: {0}")]
    KeyDerivationFailed(String),
}
