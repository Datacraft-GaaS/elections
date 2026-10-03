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

use anyhow::Result;
use tracing::{error, info};

/// Run golden verification tests against the target environment.
/// Returns 0 on success, 1 on any failure.
pub async fn run(env: Option<&str>) -> Result<i32> {
    let env = env.unwrap_or("dev");
    info!(env = %env, "running IAM verification");

    let fixtures = load_fixtures(env);
    let mut failed = 0usize;

    for fixture in &fixtures {
        match run_fixture(env, fixture).await {
            Ok(()) => info!("PASS: {}", fixture.name),
            Err(e) => {
                error!("FAIL: {} - {}", fixture.name, e);
                failed += 1;
            }
        }
    }

    if failed > 0 {
        error!("{}/{} verification checks failed", failed, fixtures.len());
        return Ok(1);
    }

    info!("all {} verification checks passed", fixtures.len());
    Ok(0)
}

struct Fixture {
    name: String,
    check: FixtureKind,
}

enum FixtureKind {
    SpiceDbPermission {
        resource_type: String,
        resource_id: String,
        subject_type: String,
        subject_id: String,
        permission: String,
        expect_allowed: bool,
    },
}

fn load_fixtures(_env: &str) -> Vec<Fixture> {
    // Golden fixtures for SpiceDB permission checks.
    // Extend with env-specific fixtures as the schema grows.
    vec![
        Fixture {
            name: "voter can read their own registration".to_string(),
            check: FixtureKind::SpiceDbPermission {
                resource_type: "voter_registration".to_string(),
                resource_id: "test-reg-001".to_string(),
                subject_type: "voter".to_string(),
                subject_id: "test-voter-001".to_string(),
                permission: "view".to_string(),
                expect_allowed: true,
            },
        },
        Fixture {
            name: "anonymous cannot modify registration".to_string(),
            check: FixtureKind::SpiceDbPermission {
                resource_type: "voter_registration".to_string(),
                resource_id: "test-reg-001".to_string(),
                subject_type: "anonymous".to_string(),
                subject_id: "anon".to_string(),
                permission: "update".to_string(),
                expect_allowed: false,
            },
        },
    ]
}

async fn run_fixture(env: &str, fixture: &Fixture) -> Result<()> {
    match &fixture.check {
        FixtureKind::SpiceDbPermission {
            resource_type,
            resource_id,
            subject_type,
            subject_id,
            permission,
            expect_allowed,
        } => {
            let endpoint = format!("spicedb.{}.eel.internal:50051", env);
            let token = std::env::var("SPICEDB_TOKEN").unwrap_or_default();
            check_spicedb_permission(
                &endpoint,
                &token,
                resource_type,
                resource_id,
                subject_type,
                subject_id,
                permission,
                *expect_allowed,
            )
            .await
        }
    }
}

async fn check_spicedb_permission(
    endpoint: &str,
    token: &str,
    resource_type: &str,
    resource_id: &str,
    subject_type: &str,
    subject_id: &str,
    permission: &str,
    expect_allowed: bool,
) -> Result<()> {
    let url = format!(
        "https://{}/v1/permissions/check",
        endpoint.trim_start_matches("https://")
    );
    let body = serde_json::json!({
        "resource": { "objectType": resource_type, "objectId": resource_id },
        "subject": { "object": { "objectType": subject_type, "objectId": subject_id } },
        "permission": permission,
    });

    let client = reqwest::Client::builder().use_rustls_tls().build()?;
    let resp = client
        .post(&url)
        .bearer_auth(token)
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;

    let allowed = resp
        .get("permissionship")
        .and_then(|v| v.as_str())
        .map(|s| s == "PERMISSIONSHIP_HAS_PERMISSION")
        .unwrap_or(false);

    if allowed != expect_allowed {
        anyhow::bail!(
            "permission check {}/{} {} {}/{}: expected allowed={}, got allowed={}",
            resource_type,
            resource_id,
            permission,
            subject_type,
            subject_id,
            expect_allowed,
            allowed
        );
    }
    Ok(())
}
