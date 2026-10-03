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

//! crms-domain: core types, quota engine, clearance FSM, and symbol checks.
//!
//! WP-4.11 — M12 (CRMS), M45 (Party Registration), M46 (Primaries Oversight),
//! M47 (Nomination Fee Payment), M67 (Symbol/Colour Registry).
#![warn(missing_docs)]

pub mod ballot;
pub mod candidate;
pub mod clearance;
pub mod coalition;
pub mod events;
pub mod gazette;
pub mod nomination;
pub mod party;
pub mod payment;
pub mod primaries;
pub mod quota;
pub mod symbol;
