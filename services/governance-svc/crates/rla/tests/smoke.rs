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

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use eel_governance_rla::store::Store;
use std::collections::HashSet;

const BEARER: &str = "Bearer test-token";
const WINNER: &str = "Ada";

fn mem_app() -> axum::Router {
    eel_governance_rla::router::build_router(
        Store::Mem(Default::default()),
        eel_governance_rla::SpecState::new(),
    )
}

fn create_body() -> serde_json::Value {
    serde_json::json!({
        "election_id": "E-2026",
        "contest_id": "C-GOV",
        "risk_limit": 0.05,
        "winner_id": WINNER,
        "loser_id": "Grace",
        "winner_votes": 600,
        "loser_votes": 400,
        "seed": "beacon-pulse-42",
        "sample_size": 25,
        "manifest": {
            "manifest_id": "m-1",
            "election_id": "E-2026",
            "total_ballots": 10_000,
            "batches": [
                { "batch_id": "BAT-000", "num_ballots": 5_000, "location": "loc-a" },
                { "batch_id": "BAT-001", "num_ballots": 5_000, "location": "loc-b" }
            ]
        }
    })
}

fn request(method: &str, uri: &str, bearer: Option<&str>, body: Option<String>) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(b) = bearer {
        builder = builder.header("authorization", b);
    }
    match body {
        Some(json) => builder
            .header("content-type", "application/json")
            .body(Body::from(json))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    }
}

async fn json_of(resp: axum::response::Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn create_run(app: &axum::Router) -> String {
    let resp = app
        .clone()
        .oneshot(request(
            "POST",
            "/rla/audits",
            Some(BEARER),
            Some(create_body().to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    assert!(resp.headers().get("location").is_some());
    let v = json_of(resp).await;
    v["run_id"].as_str().unwrap().to_string()
}

async fn sample_ids(app: &axum::Router, run_id: &str) -> Vec<String> {
    let resp = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/rla/audits/{run_id}/samples"),
            None,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let v = json_of(resp).await;
    v["samples"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["ballot_id"].as_str().unwrap().to_string())
        .collect()
}

#[tokio::test]
async fn router_smoke() {
    let app = mem_app();
    drop(app);
}

#[tokio::test]
async fn missing_bearer_is_401() {
    let app = mem_app();
    for (method, uri) in [
        ("POST", "/rla/audits"),
        ("POST", "/rla/audits/run-1/samples/BAT-000/1"),
        ("POST", "/rla/audits/run-1/finalise"),
    ] {
        let body = if uri.ends_with("/audits") {
            Some(create_body().to_string())
        } else if uri.contains("samples") {
            Some(r#"{"audited_vote":"Ada"}"#.to_string())
        } else {
            None
        };
        let resp = app
            .clone()
            .oneshot(request(method, uri, None, body))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED, "{method} {uri}");
        let v = json_of(resp).await;
        assert_eq!(v["title"], "Unauthorized");
        assert_eq!(v["type"], "https://errors.eel.internal/common/unauthorized");
    }
}

#[tokio::test]
async fn premature_finalise_is_409_with_t_statistic() {
    let app = mem_app();
    let run_id = create_run(&app).await;
    let resp = app
        .clone()
        .oneshot(request(
            "POST",
            &format!("/rla/audits/{run_id}/finalise"),
            Some(BEARER),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    let v = json_of(resp).await;
    assert_eq!(v["status"], 409);
    let detail = v["detail"].as_str().unwrap();
    assert!(detail.contains("t-statistic"), "detail: {detail}");
}

#[tokio::test]
async fn duplicate_interpretation_is_409() {
    let app = mem_app();
    let run_id = create_run(&app).await;
    let ballots = sample_ids(&app, &run_id).await;
    let ballot = ballots.first().unwrap();

    let resp = app
        .clone()
        .oneshot(request(
            "POST",
            &format!("/rla/audits/{run_id}/samples/{ballot}"),
            Some(BEARER),
            Some(format!(r#"{{"audited_vote":"{WINNER}"}}"#)),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Second interpretation attempt on the same ballot must conflict.
    let resp = app
        .clone()
        .oneshot(request(
            "POST",
            &format!("/rla/audits/{run_id}/samples/{ballot}"),
            Some(BEARER),
            Some(format!(r#"{{"audited_vote":"{WINNER}"}}"#)),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);

    // Unknown ballots are 404, not 409.
    let resp = app
        .clone()
        .oneshot(request(
            "POST",
            &format!("/rla/audits/{run_id}/samples/BAT-999/999"),
            Some(BEARER),
            Some(format!(r#"{{"audited_vote":"{WINNER}"}}"#)),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn happy_path_reaches_finalised() {
    let app = mem_app();
    let run_id = create_run(&app).await;

    // Creation put the run straight into sampling with a stored draw.
    let resp = app
        .clone()
        .oneshot(request("GET", &format!("/rla/audits/{run_id}"), None, None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let v = json_of(resp).await;
    assert_eq!(v["status"], "sampling");

    let ballots = sample_ids(&app, &run_id).await;
    assert_eq!(ballots.len(), 25);

    // Report before any interpretations: inconclusive.
    let resp = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/rla/audits/{run_id}/report"),
            None,
            None,
        ))
        .await
        .unwrap();
    let v = json_of(resp).await;
    assert_eq!(v["outcome"], "inconclusive");
    assert_eq!(v["ballots_audited"], 0);

    // First interpretation auto-transitions sampling -> auditing.
    let first = ballots.first().unwrap();
    let resp = app
        .clone()
        .oneshot(request(
            "POST",
            &format!("/rla/audits/{run_id}/samples/{first}"),
            Some(BEARER),
            Some(format!(r#"{{"audited_vote":"{WINNER}"}}"#)),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let resp = app
        .clone()
        .oneshot(request("GET", &format!("/rla/audits/{run_id}"), None, None))
        .await
        .unwrap();
    assert_eq!(json_of(resp).await["status"], "auditing");

    // Premature finalise while evidence is short.
    let resp = app
        .clone()
        .oneshot(request(
            "POST",
            &format!("/rla/audits/{run_id}/finalise"),
            Some(BEARER),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);

    for ballot in ballots.iter().skip(1) {
        let resp = app
            .clone()
            .oneshot(request(
                "POST",
                &format!("/rla/audits/{run_id}/samples/{ballot}"),
                Some(BEARER),
                Some(format!(r#"{{"audited_vote":"{WINNER}"}}"#)),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "ballot {ballot}");
    }

    // Full clean sample: BRAVO threshold reached, finalise succeeds and locks.
    let resp = app
        .clone()
        .oneshot(request(
            "POST",
            &format!("/rla/audits/{run_id}/finalise"),
            Some(BEARER),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let report = json_of(resp).await;
    assert_eq!(report["outcome"], "confirmed");
    assert_eq!(report["ballots_audited"], 25);
    let risk = report["risk_achieved"].as_f64().unwrap();
    assert!(risk <= 0.05, "risk achieved {risk}");

    let resp = app
        .clone()
        .oneshot(request("GET", &format!("/rla/audits/{run_id}"), None, None))
        .await
        .unwrap();
    assert_eq!(json_of(resp).await["status"], "finalised");

    // GET report replays the stored final report.
    let resp = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/rla/audits/{run_id}/report"),
            None,
            None,
        ))
        .await
        .unwrap();
    let v = json_of(resp).await;
    assert_eq!(v["outcome"], "confirmed");
    assert!(!v["report_id"].as_str().unwrap().is_empty());

    // Locked: no more interpretations or re-finalisation.
    let resp = app
        .clone()
        .oneshot(request(
            "POST",
            &format!("/rla/audits/{run_id}/samples/{}", ballots[0]),
            Some(BEARER),
            Some(format!(r#"{{"audited_vote":"{WINNER}"}}"#)),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    let resp = app
        .oneshot(request(
            "POST",
            &format!("/rla/audits/{run_id}/finalise"),
            Some(BEARER),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn list_and_get_endpoints_behave() {
    let app = mem_app();
    let run_id = create_run(&app).await;

    let resp = app
        .clone()
        .oneshot(request("GET", "/rla/audits", None, None))
        .await
        .unwrap();
    let v = json_of(resp).await;
    assert_eq!(v["audits"].as_array().unwrap().len(), 1);
    assert_eq!(v["audits"][0]["run_id"], run_id.as_str());

    let ballots = sample_ids(&app, &run_id).await;
    let one = ballots[0].clone();
    let resp = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/rla/audits/{run_id}/samples/{one}"),
            None,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let v = json_of(resp).await;
    assert_eq!(v["ballot_id"], one.as_str());
    assert!(v["audited_vote"].is_null());

    // Missing resources are 404 problem+json.
    for uri in [
        "/rla/audits/nope",
        "/rla/audits/nope/samples",
        "/rla/audits/nope/report",
    ] {
        let resp = app
            .clone()
            .oneshot(request("GET", uri, None, None))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND, "{uri}");
        let ct = resp
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(ct.starts_with("application/problem+json"), "{ct}");
    }

    // Invalid assertions are rejected at creation time.
    let mut bad = create_body();
    bad["winner_votes"] = serde_json::json!(400);
    bad["loser_votes"] = serde_json::json!(600);
    let resp = app
        .oneshot(request(
            "POST",
            "/rla/audits",
            Some(BEARER),
            Some(bad.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn next_block_continues_the_seeded_stream() {
    let app = mem_app();
    let run_id = create_run(&app).await;
    let drawn = sample_ids(&app, &run_id).await;
    assert_eq!(drawn.len(), 25);

    // Unknown run is problem+json 404.
    let resp = app
        .clone()
        .oneshot(request("GET", "/rla/audits/nope/next-block", None, None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(ct.starts_with("application/problem+json"), "{ct}");

    // Explicit block larger than the un-sampled manifest is 400.
    let resp = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/rla/audits/{run_id}/next-block?block_size=10001"),
            None,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Default block: min(50, remaining manifest ballots).
    let resp = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/rla/audits/{run_id}/next-block"),
            None,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let v = json_of(resp).await;
    assert_eq!(v["run_id"], run_id.as_str());
    assert_eq!(v["skip"], 25);
    assert_eq!(v["requested"], 50);
    let block_ids: Vec<String> = v["ballots"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b.as_str().unwrap().to_string())
        .collect();
    assert_eq!(block_ids.len(), 50);

    let rationale = &v["rationale"];
    let current_t = rationale["current_t"].as_f64().unwrap();
    let threshold = rationale["stop_threshold"].as_f64().unwrap();
    assert!(
        (current_t - 1.0).abs() < f64::EPSILON,
        "no interpretations yet, t must be 1"
    );
    assert!(threshold > 1.0, "risk_limit 0.05 implies threshold 20");
    assert_eq!(
        rationale["required_additional_winner_ballots"].as_u64(),
        Some(17)
    );

    // The block is the exact continuation of the seeded stream: disjoint
    // from the already-drawn ids and equal to stream positions 25..75.
    let manifest: eel_governance_rla::models::AuditManifest =
        serde_json::from_value(create_body()["manifest"].clone()).unwrap();
    let expected =
        eel_governance_rla::engine::draw_sample_positions(&manifest, "beacon-pulse-42", 75)
            .unwrap();
    let expected_tail: HashSet<String> = expected
        .iter()
        .skip(25)
        .map(|(b, p)| format!("{b}/{p}"))
        .collect();
    let drawn_set: HashSet<&String> = drawn.iter().collect();
    let block_set: HashSet<&String> = block_ids.iter().collect();
    for id in &block_ids {
        assert!(!drawn_set.contains(id), "duplicate ballot {id}");
    }
    assert_eq!(block_set.len(), expected_tail.len());
    for id in &expected_tail {
        assert!(block_set.contains(id), "missing stream ballot {id}");
    }

    // Explicit block_size is honoured verbatim.
    let resp = app
        .oneshot(request(
            "GET",
            &format!("/rla/audits/{run_id}/next-block?block_size=3"),
            None,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let v = json_of(resp).await;
    assert_eq!(v["requested"], 3);
    assert_eq!(v["ballots"].as_array().unwrap().len(), 3);
}
