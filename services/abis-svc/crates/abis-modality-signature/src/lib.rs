pub mod domain;
pub mod matcher;
pub mod parser;
pub mod quality;

pub use domain::{SignatureSample, SignatureTemplate};
pub use matcher::{dtw, match_signature, Score, DEFAULT_THRESHOLD};
pub use parser::{parse_signature, serialize_signature, SignatureError};
pub use quality::{score_quality, SignatureQuality};
