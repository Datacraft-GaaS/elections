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

//! `eel-inventory-physical-seal-binding` (HARD-16)
//!
//! Cryptographic binding of a physical seal photo to a custody-chain event.
//! The kit HSM signs a SHA-256 hash of the photo bytes bound to the seal serial,
//! then this crate verifies the signature before a CustodyEvent is accepted.
#![warn(missing_docs)]

pub mod api;
pub mod bind;
pub mod error;
pub mod seal_photo;
pub mod verify;

pub use bind::{bind_custody_event, BoundCustodyEvent};
pub use error::SealBindError;
pub use seal_photo::SealPhotoSignature;
pub use verify::verify_bound_event;
