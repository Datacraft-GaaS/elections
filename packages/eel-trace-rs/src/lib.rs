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

//! `#[trace("EC-DAT-01", "FR-M02-007")]` test marker.
//!
//! A no-op attribute macro that expands to its item unchanged. Its only role is
//! to carry the traceability IDs at the test site so the `eel-completeness
//! trace` audit can map tests to requirements. Keeping the expansion a pure
//! pass-through means the annotation never alters test behavior.
//!
//! Usage: bring the attribute into scope with `use eel_trace_rs::trace;` (or
//! reference it path-qualified) then annotate a test:
//!
//! ```ignore
//! use eel_trace_rs::trace;
//!
//! #[trace("FR-M12-008", "AC-M12-03")]
//! #[test]
//! fn two_thirds_violation_detected() { ... }
//! ```
#![warn(missing_docs)]

use proc_macro::TokenStream;

/// No-op attribute macro recording traceability IDs at a test site.
#[proc_macro_attribute]
pub fn trace(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
