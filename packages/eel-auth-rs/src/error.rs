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
pub enum AuthError {
    #[error("JWT verification failed: {0}")]
    JwtVerification(String),
    #[error("JWKS fetch failed: {0}")]
    JwksFetch(String),
    #[error("Token expired")]
    TokenExpired,
    #[error("Unknown kid: {kid}")]
    UnknownKid { kid: String },
    #[error("SpiceDB error: {0}")]
    SpiceDb(String),
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Audit sink error: {0}")]
    AuditSink(String),
    #[error("Transport error: {0}")]
    Transport(String),
}
