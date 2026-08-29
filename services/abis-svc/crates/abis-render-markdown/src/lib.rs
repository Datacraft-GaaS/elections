//! abis-render-markdown crate.
#![warn(missing_docs)]
pub mod error;
pub mod render;

pub use render::{
    render_markdown, MarkdownOptions, MarkdownOutput, MarkdownTemplate, MdSection, TableAlignment,
};
