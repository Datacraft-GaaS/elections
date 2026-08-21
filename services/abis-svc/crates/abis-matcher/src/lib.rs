//! ABIS multi-modal matching engine.
//!
//! Provides fingerprint (10-print, ISO/IEC 19794-2), face (512-d embedding),
//! and iris (2048-bit IrisCode) matching at 30M-enrolment scale.
//!
//! # Architecture
//! - `modality` -- template types and the `Template` trait
//! - `matcher`  -- per-modality matchers and multi-modal fusion
//! - `index`    -- HNSW ANN index, 256-shard hasher, Bloom-filter pruner
//! - `quality`  -- quality gates (NFIQ2 / ICAO / ISO 29794)
//! - `fairness` -- per-demographic FMR/FNMR harness (ISO 19795-10)
//! - `pad`      -- presentation attack detection interface

pub mod adjudication;
pub mod batch;
pub mod domain;
pub mod fairness;
pub mod fmr_fnmr;
pub mod index;
pub mod matcher;
pub mod modality;
pub mod pad;
pub mod pipeline;
pub mod quality;

#[cfg(any(test, feature = "mock"))]
pub mod mock_matcher;

pub use domain::*;
pub use pipeline::{MatchPipeline, Pipeline11, Pipeline1N};

#[cfg(any(test, feature = "mock"))]
pub use mock_matcher::MockMatcher;
