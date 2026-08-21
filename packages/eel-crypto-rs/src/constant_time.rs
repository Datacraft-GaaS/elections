//! Constant-time byte comparison to prevent timing side channels.
//!
//! Uses the [`subtle`] crate.  Direct `==` on secret byte slices is
//! disallowed by the Semgrep rule in `semgrep-rules/no-eq-on-secrets.yml`.
//!
//! # Example
//!
//! ```
//! use eel_crypto_rs::constant_time::ct_eq;
//! let a = b"secret value";
//! let b = b"secret value";
//! assert!(ct_eq(a, b));
//! ```

use subtle::ConstantTimeEq;

/// Compare two byte slices in constant time.
///
/// Returns `true` if the slices are equal, `false` otherwise.  The
/// comparison time does not depend on the position of the first differing
/// byte.  Slices of different lengths always return `false` in constant
/// time proportional to the shorter slice length.
///
/// # Example
///
/// ```
/// use eel_crypto_rs::constant_time::ct_eq;
/// assert!(ct_eq(b"same", b"same"));
/// assert!(!ct_eq(b"aaa", b"bbb"));
/// assert!(!ct_eq(b"short", b"longer value"));
/// ```
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        // Still constant-time in length: compare a dummy pair of equal length
        // slices then return false.
        let _ = a[..a.len().min(b.len())].ct_eq(&b[..a.len().min(b.len())]);
        return false;
    }
    a.ct_eq(b).into()
}

/// Compare two fixed-size arrays in constant time.
///
/// # Example
///
/// ```
/// use eel_crypto_rs::constant_time::ct_eq_fixed;
/// assert!(ct_eq_fixed::<32>(&[0u8; 32], &[0u8; 32]));
/// assert!(!ct_eq_fixed::<32>(&[0u8; 32], &[1u8; 32]));
/// ```
pub fn ct_eq_fixed<const N: usize>(a: &[u8; N], b: &[u8; N]) -> bool {
    a.ct_eq(b).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn equal_slices_return_true() {
        assert!(ct_eq(b"hello", b"hello"));
    }

    #[test]
    fn different_slices_return_false() {
        assert!(!ct_eq(b"hello", b"world"));
    }

    #[test]
    fn different_length_returns_false() {
        assert!(!ct_eq(b"a", b"ab"));
    }

    #[test]
    fn empty_slices_are_equal() {
        assert!(ct_eq(b"", b""));
    }

    proptest! {
        #[test]
        fn prop_ct_eq_matches_normal_eq(a in proptest::collection::vec(0u8..=255, 0..64), b in proptest::collection::vec(0u8..=255, 0..64)) {
            let ct = ct_eq(&a, &b);
            let normal = a == b;
            prop_assert_eq!(ct, normal, "ct_eq must agree with == for all inputs");
        }
    }
}
