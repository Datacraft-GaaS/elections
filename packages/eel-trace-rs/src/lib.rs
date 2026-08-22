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

use proc_macro::TokenStream;

/// No-op attribute macro recording traceability IDs at a test site.
#[proc_macro_attribute]
pub fn trace(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
