//! Voice biometric modality for the EEL ABIS system.
//!
//! Provides WAV/PCM ingest with inline radix-2 FFT and MFCC extraction,
//! cosine similarity and DTW matching, and SNR/duration quality scoring.

pub mod domain;
pub mod matcher;
pub mod parser;
pub mod quality;

pub use matcher::Score;
