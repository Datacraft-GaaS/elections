pub mod clone_detect;
pub mod form_quality_gate;
pub mod grievance;
pub mod queue;
pub mod sse_handler;
pub mod verify;

pub use clone_detect::*;
pub use grievance::*;
pub use queue::*;
pub use sse_handler::*;
pub use verify::*;

use axum::Json;
use serde_json::Value;

pub async fn health() -> Json<Value> {
	Json(serde_json::json!({"status": "ok", "service": "mobile-bff"}))
}
