pub mod domain;
pub mod matcher;
pub mod parser;
pub mod quality;

pub use domain::{LbpHistogram, PeriocularTemplate, PATCH_SIZE};
pub use matcher::{chi_square, match_periocular, Score, DEFAULT_THRESHOLD};
pub use parser::{compute_lbp_histogram, parse_periocular, serialize_periocular, PeriocularError};
pub use quality::{laplacian_variance, score_quality, PeriocularQuality};
