//! Fuzz target: verify that Debug and serde on PII types never expose the
//! raw value, regardless of what string is stored inside.

#![no_main]
use eel_pii_rs::{NoopAuditSink, PIIRestricted, PIISensitive, Purpose};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let r = PIIRestricted::new(s.to_owned());
        let dbg = format!("{r:?}");
        assert!(
            !dbg.contains(s) || s.is_empty() || s.contains("REDACTED"),
            "PIIRestricted Debug leaked value"
        );
        assert!(
            serde_json::to_string(&r).is_err(),
            "PIIRestricted must not serialize"
        );

        let sens = PIISensitive::new(s.to_owned());
        let dbg2 = format!("{sens:?}");
        assert!(
            !dbg2.contains(s) || s.is_empty() || s.contains("REDACTED"),
            "PIISensitive Debug leaked value"
        );
        assert!(
            serde_json::to_string(&sens).is_err(),
            "PIISensitive must not serialize"
        );

        // reveal must return the correct value
        let val = r.reveal(Purpose::Audit, &NoopAuditSink);
        assert_eq!(val, s);
    }
});
