//! API pull channel adapter.
//!
//! No-op sender: delivery stays pending until an eel_report_access_log row
//! lands for this artifact. The pipeline periodically reconciles api_pull
//! deliveries via repository::api_pull_observed().

use crate::domain::{DeliveryContext, DeliveryOutcome};

pub async fn deliver(ctx: DeliveryContext) -> DeliveryOutcome {
	// Signal the pipeline to keep this in pending state.
	// The pipeline checks access_log separately and calls mark_delivered.
	tracing::debug!(
		delivery_id = %ctx.delivery.delivery_id,
		artifact_id = %ctx.artifact.artifact_id,
		"api_pull: delivery remains pending until access log observed"
	);
	// Return a special signal: treat as "pending, no dispatch needed yet".
	// The pipeline interprets Retryable with this prefix as api_pull hold.
	DeliveryOutcome::Retryable {
		error: "api_pull:waiting_for_access".into(),
	}
}
