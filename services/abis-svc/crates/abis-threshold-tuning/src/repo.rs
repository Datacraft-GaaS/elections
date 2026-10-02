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

//! sqlx CRUD for eel_threshold_profiles.
//! Uses runtime query form to avoid requiring DATABASE_URL at compile time.

use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::domain::ThresholdProfile;
use crate::error::ThresholdError;

#[derive(sqlx::FromRow)]
struct ProfileRow {
	id: Uuid,
	modality: String,
	age_band: String,
	sex: String,
	county_code: i16,
	target_fmr: f64,
	threshold: f64,
	ci_lower: f64,
	ci_upper: f64,
	n_pairs: i64,
	achieved_fmr: f64,
	achieved_fnmr: f64,
	active: bool,
	created_at: chrono::DateTime<chrono::Utc>,
	note: Option<String>,
}

impl From<ProfileRow> for ThresholdProfile {
	fn from(r: ProfileRow) -> Self {
		Self {
			id: r.id,
			modality: r.modality,
			age_band: r.age_band,
			sex: r.sex,
			county_code: r.county_code,
			target_fmr: r.target_fmr,
			threshold: r.threshold,
			ci_lower: r.ci_lower,
			ci_upper: r.ci_upper,
			n_pairs: r.n_pairs,
			achieved_fmr: r.achieved_fmr,
			achieved_fnmr: r.achieved_fnmr,
			active: r.active,
			created_at: r.created_at,
			note: r.note,
		}
	}
}

pub struct ThresholdRepo {
	pool: PgPool,
}

impl ThresholdRepo {
	pub fn new(pool: PgPool) -> Self {
		Self { pool }
	}

	#[instrument(skip(self))]
	pub async fn insert(&self, p: &ThresholdProfile) -> Result<Uuid, ThresholdError> {
		let id: Uuid = sqlx::query_scalar(
			r#"
			INSERT INTO eel_threshold_profiles
			    (id, modality, age_band, sex, county_code, target_fmr, threshold,
			     ci_lower, ci_upper, n_pairs, achieved_fmr, achieved_fnmr, active, created_at, note)
			VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
			RETURNING id
			"#,
		)
		.bind(p.id)
		.bind(&p.modality)
		.bind(&p.age_band)
		.bind(&p.sex)
		.bind(p.county_code)
		.bind(p.target_fmr)
		.bind(p.threshold)
		.bind(p.ci_lower)
		.bind(p.ci_upper)
		.bind(p.n_pairs)
		.bind(p.achieved_fmr)
		.bind(p.achieved_fnmr)
		.bind(p.active)
		.bind(p.created_at)
		.bind(p.note.as_deref())
		.fetch_one(&self.pool)
		.await?;
		Ok(id)
	}

	#[instrument(skip(self))]
	pub async fn get(&self, id: Uuid) -> Result<ThresholdProfile, ThresholdError> {
		let row: Option<ProfileRow> = sqlx::query_as(
			r#"SELECT id, modality, age_band, sex, county_code, target_fmr, threshold,
			          ci_lower, ci_upper, n_pairs, achieved_fmr, achieved_fnmr, active, created_at, note
			   FROM eel_threshold_profiles WHERE id = $1"#,
		)
		.bind(id)
		.fetch_optional(&self.pool)
		.await?;
		row.map(ThresholdProfile::from).ok_or(ThresholdError::NotFound(id))
	}

	#[instrument(skip(self))]
	pub async fn list(&self) -> Result<Vec<ThresholdProfile>, ThresholdError> {
		let rows: Vec<ProfileRow> = sqlx::query_as(
			r#"SELECT id, modality, age_band, sex, county_code, target_fmr, threshold,
			          ci_lower, ci_upper, n_pairs, achieved_fmr, achieved_fnmr, active, created_at, note
			   FROM eel_threshold_profiles ORDER BY created_at DESC"#,
		)
		.fetch_all(&self.pool)
		.await?;
		Ok(rows.into_iter().map(ThresholdProfile::from).collect())
	}

	#[instrument(skip(self))]
	pub async fn delete(&self, id: Uuid) -> Result<(), ThresholdError> {
		sqlx::query("DELETE FROM eel_threshold_profiles WHERE id = $1")
			.bind(id)
			.execute(&self.pool)
			.await?;
		Ok(())
	}
}
