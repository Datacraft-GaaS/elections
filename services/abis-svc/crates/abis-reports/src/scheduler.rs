use chrono::{DateTime, Utc};
use croner::Cron;
use sqlx::PgPool;
use tracing::{error, info, warn};

use crate::repository;

/// Start the scheduler loop. Scans active definitions with a cron schedule,
/// computes next-fire time, and enqueues runs idempotently.
pub async fn scheduler_loop(pool: PgPool, cancel: tokio_util::sync::CancellationToken) {
	info!("scheduler loop started");

	// Log startup summary
	match repository::fetch_active_definitions(&pool, None).await {
		Ok(defs) => {
			for def in &defs {
				if let Some(sched) = &def.default_schedule {
					match next_fire(sched) {
						Ok(Some(t)) => info!(
							slug = %def.slug,
							schedule = %sched,
							next_fire = %t,
							"scheduler: next fire"
						),
						Ok(None) => {}
						Err(e) => warn!(slug = %def.slug, error = %e, "invalid cron expression"),
					}
				}
			}
		}
		Err(e) => error!(error = %e, "scheduler: failed to load definitions"),
	}

	let requested_by = "scheduler";

	while !cancel.is_cancelled() {
		match repository::fetch_active_definitions(&pool, None).await {
			Ok(defs) => {
				let now = Utc::now();
				for def in defs {
					let Some(sched) = &def.default_schedule else { continue };
					match next_fire_at_or_before(sched, now) {
						Ok(Some(_fire_time)) => {
							// Window: previous fire to now (approximate)
							let window_from = now - chrono::Duration::hours(1);
							let window_to = now;
							if let Err(e) = repository::create_run(
								&pool,
								def.definition_id,
								&def.slug,
								window_from,
								window_to,
								requested_by,
								None,
								None,
							)
							.await
							{
								// Idempotency conflict is expected — not an error
								if !e.to_string().contains("unique") {
									error!(slug = %def.slug, error = %e, "scheduler: create_run failed");
								}
							}
						}
						Ok(None) => {}
						Err(e) => warn!(slug = %def.slug, error = %e, "scheduler: cron parse error"),
					}
				}
			}
			Err(e) => error!(error = %e, "scheduler: failed to load definitions"),
		}

		// Check every 30 seconds
		tokio::time::sleep(std::time::Duration::from_secs(30)).await;
	}

	info!("scheduler loop stopped");
}

/// Returns the next fire time after `now` for a cron expression.
pub fn next_fire(cron_str: &str) -> Result<Option<DateTime<Utc>>, String> {
	let cron = Cron::new(cron_str)
		.with_seconds_optional()
		.parse()
		.map_err(|e| format!("cron parse error: {e}"))?;

	let next = cron
		.find_next_occurrence(&Utc::now(), false)
		.map_err(|e| format!("cron next occurrence: {e}"))?;
	Ok(Some(next))
}

/// Returns Some if the most recent past fire would have been within the last
/// scheduling window (i.e. we should fire now).
fn next_fire_at_or_before(
	cron_str: &str,
	_now: DateTime<Utc>,
) -> Result<Option<DateTime<Utc>>, String> {
	// For simplicity: check if next fire is within 60s (i.e. due right now)
	let cron = Cron::new(cron_str)
		.with_seconds_optional()
		.parse()
		.map_err(|e| format!("cron parse: {e}"))?;

	let next = cron
		.find_next_occurrence(&Utc::now(), false)
		.map_err(|e| format!("cron next: {e}"))?;

	let diff = (next - Utc::now()).num_seconds().abs();
	if diff <= 60 {
		Ok(Some(next))
	} else {
		Ok(None)
	}
}
