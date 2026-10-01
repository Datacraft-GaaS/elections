//! Per-station queue state.
//!
//! `get_queue_snapshot` used to live here and returned `queue_length: 0` for any
//! station, which reads as "no one is waiting" for every station in the country.
//! The real one is in [`crate::handlers::verify`], reading the store that agents
//! check in against; this module keeps the queue module's namespace.

pub use crate::handlers::verify::{get_queue_snapshot, queue_check_in, queue_check_out};
