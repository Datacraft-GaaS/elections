//! tlog-witness — witness cosign API and IAB witness client.

pub mod handler;
pub mod iab_client;
pub mod publisher;
pub mod split_view;

pub use handler::WitnessHandler;
pub use publisher::{NoopPublisher, WitnessPublisher};
pub use split_view::SplitViewDetector;
