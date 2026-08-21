//! Iris template encoding: Daugman rubber-sheet normalisation + 2D Gabor IrisCode.
//!
//! References:
//!   Daugman, J. (1993). High confidence visual recognition of persons by a test of statistical independence.
//!   IEEE TPAMI, 15(11), 1148-1161.
//!
//!   Daugman, J. (2004). How iris recognition works.
//!   IEEE Transactions on Circuits and Systems for Video Technology, 14(1), 21-30.
//!
//!   Prickle Interpolated Encoding (PIE): internal variant for rotation-robust encoding.

pub mod daugman_code;
pub mod encode;
pub mod gabor;
pub mod normalise;
pub mod pie;
pub mod serde_array;

pub use encode::{EncodeError, IrisEncoder, IrisTemplate};

/// Normalised iris image dimensions (Daugman rubber-sheet standard).
pub const NORM_WIDTH: u32 = 512;
pub const NORM_HEIGHT: u32 = 64;

/// IrisCode length in bits (2048-bit standard).
pub const IRIS_CODE_BITS: usize = 2048;
/// IrisCode length in bytes.
pub const IRIS_CODE_BYTES: usize = IRIS_CODE_BITS / 8;
