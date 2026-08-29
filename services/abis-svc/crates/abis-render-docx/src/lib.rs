//! abis-render-docx crate.
#![warn(missing_docs)]
pub mod error;
pub mod render;

pub use render::{
    render_docx, CalloutStyle, DocxOptions, DocxOutput, DocxTemplate, PageOrientation, PageSize,
    Run, Section,
};
