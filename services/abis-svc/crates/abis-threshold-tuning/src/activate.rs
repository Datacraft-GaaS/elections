// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Atomic activation of a threshold profile (only one active per cohort).

use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::error::ThresholdError;

/// Atomically activate a profile: sets active=TRUE on the target and FALSE on all siblings.
#[instrument(skip(pool))]
pub async fn activate_profile(pool: &PgPool, id: Uuid) -> Result<(), ThresholdError> {
	let mut tx = pool.begin().await?;

	sqlx::query(
		r#"
		UPDATE eel_threshold_profiles
		   SET active = FALSE
		 WHERE (modality, age_band, sex, county_code) = (
		       SELECT modality, age_band, sex, county_code
		         FROM eel_threshold_profiles
		        WHERE id = $1
		       )
		"#,
	)
	.bind(id)
	.execute(&mut *tx)
	.await?;

	let affected = sqlx::query("UPDATE eel_threshold_profiles SET active = TRUE WHERE id = $1")
		.bind(id)
		.execute(&mut *tx)
		.await?
		.rows_affected();

	if affected == 0 {
		tx.rollback().await?;
		return Err(ThresholdError::NotFound(id));
	}

	tx.commit().await?;
	Ok(())
}
