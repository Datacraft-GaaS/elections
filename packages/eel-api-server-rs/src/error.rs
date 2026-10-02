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
pub enum ApiServerError {
    #[error("observability init failed: {0}")]
    Observability(#[from] eel_observability_rs::ObservabilityError),

    #[error("auth configuration error: {0}")]
    Auth(String),

    #[error("server bind error: {0}")]
    Bind(#[from] std::io::Error),

    #[error("server error: {0}")]
    Runtime(String),
}
