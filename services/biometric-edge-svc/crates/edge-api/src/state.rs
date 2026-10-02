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

/// Shared application state injected into axum handlers via Extension.
#[derive(Clone, Debug)]
pub struct AppState {
    /// Base URL of bvr-svc for enrolment proxying.
    pub bvr_svc_url: String,
    /// Base URL of bvi-svc for config bundle download.
    pub bvi_svc_url: String,
    /// Base URL of ECC for distress alert routing.
    pub ecc_svc_url: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            bvr_svc_url: std::env::var("BVR_SVC_URL")
                .unwrap_or_else(|_| "http://bvr-svc.eel.local:8080".into()),
            bvi_svc_url: std::env::var("BVI_SVC_URL")
                .unwrap_or_else(|_| "http://bvi-svc.eel.local:8081".into()),
            ecc_svc_url: std::env::var("ECC_SVC_URL")
                .unwrap_or_else(|_| "http://ecc-svc.eel.local:8082".into()),
        }
    }
}
