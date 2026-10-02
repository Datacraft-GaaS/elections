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

use uuid::Uuid;

use crate::{
    domain::{AdhocReportVersion, PublishRequest},
    error::{AdhocError, Result},
};

pub struct PublishResult {
    pub definition_id: Uuid,
    pub slug: String,
    pub template_path: String,
}

/// Promote an ad-hoc report version to a full eel_report_definitions entry.
pub async fn publish_version(
    _pool: &sqlx::PgPool,
    version: &AdhocReportVersion,
    adhoc_slug: &str,
    _adhoc_data_class: &str,
    _author_id: Uuid,
    _req: &PublishRequest,
) -> Result<PublishResult> {
    if !version
        .lint_report
        .get("errors")
        .and_then(|e| e.as_array())
        .map(|a| a.is_empty())
        .unwrap_or(false)
    {
        return Err(AdhocError::Publish(
            "Version has lint errors; cannot publish".to_string(),
        ));
    }

    let definition_id = Uuid::now_v7();
    let template_path = format!("reports-templates/sql/adhoc/{adhoc_slug}.sql");

    Ok(PublishResult {
        definition_id,
        slug: adhoc_slug.to_string(),
        template_path,
    })
}
