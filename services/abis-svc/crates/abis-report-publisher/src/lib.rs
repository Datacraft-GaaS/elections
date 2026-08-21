//! Report publication and delivery for the ABIS service (RPT-B).
//!
//! Orchestrates delivery of generated report artefacts to subscribers via:
//! email, webhook, kafka, sftp, gov_printer, kbc_sftp, and api_pull channels.

pub mod domain;
pub mod repository;
pub mod channels;
pub mod pipeline;
pub mod api;
pub mod metrics;
