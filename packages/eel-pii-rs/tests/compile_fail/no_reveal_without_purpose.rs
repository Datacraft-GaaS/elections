//! Compile-fail: calling reveal() without a Purpose must not compile.
//! This file is expected to fail with a type error.

use eel_pii_rs::{NoopAuditSink, PIIRestricted};

fn main() {
    let v = PIIRestricted::new("secret".to_string());
    // The following line should fail: reveal() requires a Purpose argument.
    let _ = v.reveal(&NoopAuditSink); //~ ERROR
}
