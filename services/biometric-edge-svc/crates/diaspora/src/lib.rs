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

//! Diaspora voting domain, HTTP API, and transmission pipeline (WP-5.8, M24).
//!
//! Covers:
//!   - Embassy station registry (open/close lifecycle)
//!   - Diaspora roll management (import, freeze, hash)
//!   - Form 34A capture and HSM signing
//!   - Physical pouch custody chain
//!   - Delayed batch transmission with exponential backoff
//!   - CloudEvent emission via eel-outbox-rs
#![warn(missing_docs)]

pub mod api;
pub mod domain;
pub mod events;
pub mod import;
pub mod roll;
pub mod spec;
pub mod store;
pub mod transmission;
pub mod tz;

pub use domain::{CustodyEvent, DiasporaRoll, EmbassyStation, Form34ADiaspora, PouchCustody};
