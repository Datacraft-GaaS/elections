pub mod domain;
pub mod matcher;
pub mod parser;
pub mod quality;

pub use domain::{BodySide, VeinTemplate, VeinType};
pub use matcher::{match_vein, ncc, Score, DEFAULT_THRESHOLD};
pub use parser::{parse_nir_bitmap, serialize_nir_bitmap, VeinError};
pub use quality::{count_branch_points, score_quality, VeinQuality};
