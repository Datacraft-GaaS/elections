/// Normalize a string to Unicode NFC form.
/// Passthrough stub; production should use the `unicode-normalization` crate.
/// NFC normalization is idempotent on ASCII, which covers most BVR data.
#[allow(dead_code)]
pub fn nfc(s: impl Into<String>) -> String {
    s.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nfc_passthrough_ascii() {
        assert_eq!(nfc("hello"), "hello");
    }
}
