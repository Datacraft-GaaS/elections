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

//! Gender-quota handlers.

use super::*;

/// Evaluate the two-thirds gender quota.
pub async fn evaluate_quota_rule(
    Json(req): Json<EvaluateQuota>,
) -> Result<Json<QuotaStatus>> {
    let roster: Vec<(crms_domain::candidate::Gender, bool, &str)> = req
        .candidates
        .iter()
        .map(|c| (c.gender.clone(), c.is_pwd, c.date_of_birth.as_str()))
        .collect();
    let status = evaluate_quota(req.scope, req.position, &roster, &req.nomination_date);
    if !status.two_thirds_compliant {
        stateless_quota_event(&status);
    }
    Ok(Json(status))
}

fn stateless_quota_event(_status: &QuotaStatus) {
    // Quota evaluation is stateless; consumers read the flag from the response.
    // A wired deployment forwards this to the telemetry/ECC sink.
    let _ = _status;
}
