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

use eel_abis_reports_adhoc::{
    domain::*,
    query_builder::{BuilderQuery, JoinClause, JoinKind, OrderByClause, QueryBuilder, SqlQuery},
    validator::{validate_sql, BLOCKED_FUNCTIONS, DEFAULT_MAX_ROWS},
};
use serde_json::json;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Validator tests
// ---------------------------------------------------------------------------

fn allowed() -> Vec<String> {
    vec![
        "eel_voters".to_string(),
        "eel_polling_stations".to_string(),
        "eel_results".to_string(),
    ]
}

#[test]
fn validator_rejects_delete() {
    let r = validate_sql("DELETE FROM eel_voters WHERE id = 1", &allowed(), None).unwrap();
    assert!(!r.is_safe());
    assert!(r.errors.iter().any(|e| e.code == "E003"));
}

#[test]
fn validator_rejects_insert() {
    let r = validate_sql("INSERT INTO eel_voters VALUES (1)", &allowed(), None).unwrap();
    assert!(!r.is_safe());
}

#[test]
fn validator_rejects_update() {
    let r = validate_sql("UPDATE eel_voters SET name = 'x'", &allowed(), None).unwrap();
    assert!(!r.is_safe());
}

#[test]
fn validator_rejects_truncate() {
    let r = validate_sql("TRUNCATE TABLE eel_voters", &allowed(), None).unwrap();
    assert!(!r.is_safe());
}

#[test]
fn validator_rejects_vacuum() {
    let r = validate_sql("VACUUM eel_voters", &allowed(), None).unwrap();
    assert!(!r.is_safe());
}

#[test]
fn validator_rejects_alter() {
    let r = validate_sql("ALTER TABLE eel_voters ADD COLUMN x TEXT", &allowed(), None).unwrap();
    assert!(!r.is_safe());
}

#[test]
fn validator_rejects_copy() {
    let r = validate_sql("COPY eel_voters TO '/tmp/out.csv'", &allowed(), None).unwrap();
    assert!(!r.is_safe());
}

#[test]
fn validator_rejects_do_block() {
    let r = validate_sql("DO $$ BEGIN RAISE NOTICE 'x'; END $$", &allowed(), None).unwrap();
    assert!(!r.is_safe());
}

#[test]
fn validator_rejects_unlisted_table() {
    let r = validate_sql("SELECT * FROM eel_adjudication_cases", &allowed(), None).unwrap();
    assert!(!r.is_safe());
    assert!(r.errors.iter().any(|e| e.code == "E005"));
}

#[test]
fn validator_rejects_pg_read_file() {
    let r = validate_sql("SELECT pg_read_file('/etc/passwd')", &[], None).unwrap();
    assert!(!r.is_safe());
    assert!(r.errors.iter().any(|e| e.code == "E004"));
}

#[test]
fn validator_rejects_dblink() {
    let r = validate_sql(
        "SELECT * FROM dblink('host=other', 'SELECT 1') AS t(id int)",
        &[],
        None,
    )
    .unwrap();
    assert!(!r.is_safe());
}

#[test]
fn validator_injects_limit_when_missing() {
    let r = validate_sql(
        "SELECT * FROM eel_voters",
        &["eel_voters".to_string()],
        None,
    )
    .unwrap();
    assert!(r.modified_sql.is_some());
    let modified = r.modified_sql.unwrap();
    assert!(modified.to_uppercase().contains("LIMIT"));
}

#[test]
fn validator_does_not_inject_limit_when_present() {
    let r = validate_sql(
        "SELECT * FROM eel_voters LIMIT 50",
        &["eel_voters".to_string()],
        None,
    )
    .unwrap();
    assert!(r.modified_sql.is_none());
}

#[test]
fn validator_safety_score_decreases_with_join_depth() {
    let simple = validate_sql(
        "SELECT * FROM eel_voters LIMIT 100",
        &["eel_voters".to_string()],
        None,
    )
    .unwrap();

    let joined = validate_sql(
        "SELECT v.*, p.name FROM eel_voters v \
         JOIN eel_polling_stations p ON v.station_id = p.id \
         JOIN eel_results r ON r.voter_id = v.id \
         JOIN eel_results r2 ON r2.voter_id = v.id \
         JOIN eel_results r3 ON r3.voter_id = v.id \
         LIMIT 100",
        &allowed(),
        None,
    )
    .unwrap();

    assert!(joined.safety_score <= simple.safety_score);
}

#[test]
fn validator_rejects_system_catalog() {
    let r = validate_sql("SELECT * FROM pg_class", &[], None).unwrap();
    assert!(!r.is_safe());
    assert!(r.errors.iter().any(|e| e.code == "E006"));
}

// ---------------------------------------------------------------------------
// QueryBuilder tests
// ---------------------------------------------------------------------------

#[test]
fn query_builder_sql_passthrough() {
    let q = QueryBuilder::Sql(SqlQuery {
        text: "SELECT 1".to_string(),
        parameters: vec![],
    });
    let compiled = q.compile(&[]).unwrap();
    assert_eq!(compiled.text, "SELECT 1");
}

#[test]
fn query_builder_builder_compiles_to_sql() {
    let q = QueryBuilder::Builder(BuilderQuery {
        source_tables: vec!["eel_voters".to_string()],
        projections: vec!["id".to_string(), "name".to_string()],
        filters: vec!["archived_at IS NULL".to_string()],
        joins: vec![],
        group_by: vec![],
        order_by: vec![OrderByClause {
            column: "name".to_string(),
            descending: false,
        }],
        limit: Some(100),
    });
    let compiled = q.compile(&["eel_voters".to_string()]).unwrap();
    assert!(compiled.text.contains("SELECT id, name FROM eel_voters"));
    assert!(compiled.text.contains("WHERE archived_at IS NULL"));
    assert!(compiled.text.contains("LIMIT 100"));
}

#[test]
fn query_builder_rejects_unlisted_table() {
    let q = QueryBuilder::Builder(BuilderQuery {
        source_tables: vec!["eel_voters".to_string()],
        projections: vec![],
        filters: vec![],
        joins: vec![],
        group_by: vec![],
        order_by: vec![],
        limit: None,
    });
    let result = q.compile(&["eel_results".to_string()]);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Domain tests
// ---------------------------------------------------------------------------

#[test]
fn create_report_request_serialises() {
    let req = CreateReportRequest {
        slug: "test-report".to_string(),
        title: "Test Report".to_string(),
        description: Some("A test".to_string()),
        data_class: DataClass::Sensitive,
        visibility: Visibility::Private,
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("test-report"));
}

#[test]
fn preview_request_sample_limit_default() {
    let req = PreviewRequest {
        parameter_values: None,
        sample_limit: None,
        format: None,
        idempotency_key: None,
    };
    assert!(req.sample_limit.is_none());
}

#[test]
fn all_blocked_functions_covered() {
    for fn_name in BLOCKED_FUNCTIONS {
        let sql = format!("SELECT {fn_name}('/tmp/x')");
        let r = validate_sql(&sql, &[], None).unwrap();
        assert!(
            !r.is_safe() || r.errors.iter().any(|e| e.code == "E004"),
            "Expected {fn_name} to be blocked"
        );
    }
}

// ---------------------------------------------------------------------------
// Preview tests (in-memory)
// ---------------------------------------------------------------------------

async fn make_rows_future(
    n: usize,
) -> eel_abis_reports_adhoc::error::Result<Vec<serde_json::Value>> {
    Ok((0..n)
        .map(|i| json!({"id": i, "name": format!("row_{i}")}))
        .collect())
}

#[test]
fn preview_refuses_over_1000_sample_limit() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let version = make_test_version();
        let req = PreviewRequest {
            parameter_values: None,
            sample_limit: Some(1001),
            format: Some(PreviewFormat::Json),
            idempotency_key: None,
        };
        let result = eel_abis_reports_adhoc::preview::run_preview(
            &version,
            req,
            Uuid::now_v7(),
            make_rows_future(0),
        )
        .await;
        assert!(result.is_err());
    });
}

#[test]
fn preview_json_format_returns_bytes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let version = make_test_version();
        let req = PreviewRequest {
            parameter_values: None,
            sample_limit: Some(10),
            format: Some(PreviewFormat::Json),
            idempotency_key: None,
        };
        let result = eel_abis_reports_adhoc::preview::run_preview(
            &version,
            req,
            Uuid::now_v7(),
            make_rows_future(5),
        )
        .await;
        assert!(result.is_ok());
        let preview = result.unwrap();
        assert!(preview.preview_bytes.is_some());
    });
}

#[test]
fn preview_markdown_format_returns_bytes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let version = make_test_version();
        let req = PreviewRequest {
            parameter_values: None,
            sample_limit: Some(10),
            format: Some(PreviewFormat::Markdown),
            idempotency_key: None,
        };
        let result = eel_abis_reports_adhoc::preview::run_preview(
            &version,
            req,
            Uuid::now_v7(),
            make_rows_future(5),
        )
        .await;
        assert!(result.is_ok());
    });
}

#[test]
fn preview_docx_format_returns_bytes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let version = make_test_version();
        let req = PreviewRequest {
            parameter_values: None,
            sample_limit: Some(10),
            format: Some(PreviewFormat::Docx),
            idempotency_key: None,
        };
        let result = eel_abis_reports_adhoc::preview::run_preview(
            &version,
            req,
            Uuid::now_v7(),
            make_rows_future(5),
        )
        .await;
        assert!(result.is_ok());
    });
}

#[test]
fn preview_csv_format_returns_bytes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let version = make_test_version();
        let req = PreviewRequest {
            parameter_values: None,
            sample_limit: Some(10),
            format: Some(PreviewFormat::Csv),
            idempotency_key: None,
        };
        let result = eel_abis_reports_adhoc::preview::run_preview(
            &version,
            req,
            Uuid::now_v7(),
            make_rows_future(5),
        )
        .await;
        assert!(result.is_ok());
    });
}

#[test]
fn preview_expires_after_2h_in_memory() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let version = make_test_version();
        let req = PreviewRequest {
            parameter_values: None,
            sample_limit: Some(10),
            format: Some(PreviewFormat::Json),
            idempotency_key: None,
        };
        let preview = eel_abis_reports_adhoc::preview::run_preview(
            &version,
            req,
            Uuid::now_v7(),
            make_rows_future(3),
        )
        .await
        .unwrap();

        let now = chrono::Utc::now();
        let expires_in = preview.expires_at - now;
        assert!(expires_in.num_minutes() > 110 && expires_in.num_minutes() < 125);
    });
}

fn make_test_version() -> AdhocReportVersion {
    AdhocReportVersion {
        version_id: Uuid::now_v7(),
        adhoc_id: Uuid::now_v7(),
        version_number: 1,
        source_type: SourceType::Sql,
        source: "SELECT * FROM eel_voters LIMIT 10".to_string(),
        parameters: json!([]),
        allowed_tables: vec!["eel_voters".to_string()],
        column_projection: None,
        render_pipeline: vec!["json".to_string()],
        created_at: chrono::Utc::now(),
        created_by: Uuid::now_v7(),
        commit_message: None,
        linted_at: None,
        lint_report: json!({"errors": [], "warnings": []}),
        safety_score: 0.9,
    }
}
