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

use crate::handlers::*;
use axum::{
    routing::{get, post},
    Router,
};

pub fn build_router() -> Router {
    Router::new()
        .route("/health", get(health))
        // M08: form 34A upload (multipart: image + OCR JSON + signature)
        .route("/api/v1/rts/device/form34a/upload", post(upload_form34a))
        // M08: SMS/USSD fallback (CBOR+zstd compressed; <= 5 SMS segments / 10-min 3G budget)
        .route(
            "/api/v1/rts/device/form34a/sms-fallback",
            post(sms_fallback),
        )
        // M08: kit reports transmission attempt/success/failure to ledger
        .route(
            "/api/v1/rts/device/transmission-attempt",
            post(transmission_attempt),
        )
        // M08: bounded random participation token for authenticated voter
        .route(
            "/api/v1/rts/device/participation-tokens/{station}",
            get(participation_tokens),
        )
        // M08: idempotent retransmission after SIM failover (same Idempotency-Key)
        .route(
            "/api/v1/rts/device/dual-sim-resubmit",
            post(dual_sim_resubmit),
        )
}
