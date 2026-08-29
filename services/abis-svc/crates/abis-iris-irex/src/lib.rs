//! NIST IREX (Iris Exchange) evaluation framework.
//!
//! Implements corpus management, 1:1 verification, 1:N identification, and
//! report-card generation aligned with IREX I through IREX X.
//!
//! Reference: <https://www.nist.gov/programs-projects/iris-exchange-irex-overview>
#![warn(missing_docs)]

pub mod corpus;
pub mod identify;
pub mod report;
pub mod verify;

pub use corpus::{CorpusEntry, CorpusError, IrexCorpus};
pub use identify::{IdentifyResult, IrexIdentifier};
pub use report::{FairnessSlice, ReportCard, RocPoint};
pub use verify::{IrexVerifier, VerifyResult};
