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

pub mod airline_double;
pub mod kra_double;
pub mod mfa_double;

#[derive(Debug, Error)]
pub enum DiasporaLogisticsError {
    #[error("MFA preclearance failed: {0}")]
    MfaPreclearance(String),
    #[error("KRA eclearance failed: {0}")]
    KraEclearance(String),
    #[error("Airline status unavailable: {0}")]
    AirlineStatus(String),
}
