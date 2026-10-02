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

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Debug)]
pub struct FeatureFlags {
    pub captcha_enabled: AtomicBool,
    pub iprs_circuit_closed: AtomicBool,
}

impl FeatureFlags {
    pub fn new(captcha_enabled: bool, iprs_circuit_closed: bool) -> Self {
        Self {
            captcha_enabled: AtomicBool::new(captcha_enabled),
            iprs_circuit_closed: AtomicBool::new(iprs_circuit_closed),
        }
    }

    pub fn is_captcha_enabled(&self) -> bool {
        self.captcha_enabled.load(Ordering::Relaxed)
    }

    pub fn is_iprs_circuit_closed(&self) -> bool {
        self.iprs_circuit_closed.load(Ordering::Relaxed)
    }
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub feature_flags: Arc<FeatureFlags>,
}

impl AppState {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self {
            db,
            feature_flags: Arc::new(FeatureFlags::new(true, false)),
        }
    }
}
