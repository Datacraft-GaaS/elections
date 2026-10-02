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
pub enum VendorError {
    #[error("extraction failed: {0}")]
    ExtractionFailed(String),
    #[error("match failed: {0}")]
    MatchFailed(String),
    #[error("vendor unavailable: {0}")]
    Unavailable(String),
    #[error("modality mismatch: probe is {probe} but gallery is {gallery}")]
    ModalityMismatch { probe: String, gallery: String },
    #[error("HTTP error from vendor bridge: {0}")]
    Http(String),
    #[error("gRPC error from vendor sidecar: {0}")]
    Grpc(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("circuit breaker open: vendor {0} is unavailable")]
    CircuitOpen(String),
}

impl From<reqwest::Error> for VendorError {
    fn from(e: reqwest::Error) -> Self {
        VendorError::Http(e.to_string())
    }
}

impl From<serde_json::Error> for VendorError {
    fn from(e: serde_json::Error) -> Self {
        VendorError::Serialization(e.to_string())
    }
}
