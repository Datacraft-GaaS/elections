//! Native Rust reimplementation of the SourceAFIS fingerprint matching algorithm.
//!
//! SourceAFIS is authored by Robert Vaszary (Rob Wozniak) and is published under
//! MPL-2.0. Algorithm documentation: <https://sourceafis.machinezoo.com/algorithm>.
//! This crate implements the same algorithm in pure Rust for the eel ABIS pipeline.
//!
//! # Algorithm overview
//! 1. Skeleton extraction via ridge-width-aware thinning (differs from Zhang-Suen by
//!    sampling ridge width before thinning so short spurs are suppressed correctly).
//! 2. Minutia detection: ridge endings and bifurcations at skeleton branch points.
//! 3. Local ridge structure descriptor: for each minutia, collect nearby minutiae and
//!    encode their relative position, direction, and inter-minutia ridge counts into a
//!    compact descriptor.
//! 4. Matching: seed-pair selection (highest descriptor similarity), radial extension
//!    from seed, score aggregation from paired-minutia count and descriptor agreement.

pub mod matcher;
pub mod minutia;
pub mod params;
pub mod ridge_structure;
pub mod skeleton;

pub use matcher::{MatchScore, SourceAfisMatcher};
pub use minutia::{MinutiaType, SourceAfisMinutia};
pub use params::SourceAfisParams;

/// Gray-scale fingerprint image (8-bit pixels, row-major).
#[derive(Debug, Clone)]
pub struct GrayImage {
	pub pixels: Vec<u8>,
	pub width: u32,
	pub height: u32,
}

impl GrayImage {
	/// Construct from raw bytes.  Panics in debug builds if `pixels.len() != width * height`.
	pub fn new(pixels: Vec<u8>, width: u32, height: u32) -> Self {
		debug_assert_eq!(pixels.len(), (width * height) as usize);
		Self { pixels, width, height }
	}

	pub fn pixel(&self, x: u32, y: u32) -> u8 {
		self.pixels[(y * self.width + x) as usize]
	}
}

/// Top-level error type.
#[derive(Debug, thiserror::Error)]
pub enum SourceAfisError {
	#[error("image too small: {0}x{1}")]
	ImageTooSmall(u32, u32),
	#[error("minutia extraction failed: {0}")]
	ExtractionFailed(String),
}
