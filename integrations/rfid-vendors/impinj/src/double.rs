//! Impinj in-memory double for dev and chaos testing.

#[path = "../double/mock_impinj.rs"]
mod mock_impinj;

pub use mock_impinj::MockImpinj;
