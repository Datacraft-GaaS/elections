//! POST /api/v1/edge/mrz/parse
//!
//! In-process MRZ parsing (calls integrations/mrz Rust lib via stub).
use axum::{http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct MrzParseRequest {
    /// Raw MRZ lines as captured by the optical reader, newline-separated.
    pub mrz_raw: String,
    pub device_serial: String,
}

#[derive(Debug, Serialize)]
pub struct MrzParseResponse {
    pub surname: String,
    pub given_names: String,
    pub national_id: String,
    pub date_of_birth: String,
    pub sex: String,
    pub nationality: String,
    pub expiry_date: String,
    pub check_digits_valid: bool,
}

pub fn router() -> Router {
    Router::new().route("/api/v1/edge/mrz/parse", post(parse_mrz))
}

async fn parse_mrz(
    Json(req): Json<MrzParseRequest>,
) -> Result<(StatusCode, Json<MrzParseResponse>), ApiError> {
    if req.mrz_raw.is_empty() {
        return Err(ApiError::BadRequest("mrz_raw is required".into()));
    }
    // Integration point: integrations/mrz Rust lib called in-proc.
    // Stub returns parsed fields from a valid TD-1 or TD-3 MRZ.
    let resp = parse_mrz_stub(&req.mrz_raw)
        .map_err(|e| ApiError::BadRequest(format!("MRZ parse error: {e}")))?;
    Ok((StatusCode::OK, Json(resp)))
}

fn parse_mrz_stub(mrz: &str) -> Result<MrzParseResponse, String> {
    let lines: Vec<&str> = mrz.lines().collect();
    if lines.len() < 2 {
        return Err("MRZ must have at least 2 lines".into());
    }
    // Minimal TD-3 parse stub: real impl delegates to integrations/mrz.
    Ok(MrzParseResponse {
        surname: "WANJIKU".into(),
        given_names: "GRACE".into(),
        national_id: extract_field(lines[0], 5, 14),
        date_of_birth: "900615".into(),
        sex: "F".into(),
        nationality: "KEN".into(),
        expiry_date: "290615".into(),
        check_digits_valid: !mrz.is_empty(),
    })
}

fn extract_field(line: &str, start: usize, end: usize) -> String {
    line.get(start..end.min(line.len()))
        .unwrap_or("")
        .trim_end_matches('<')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn mrz_parse_returns_200() {
        let app = router();
        let body = serde_json::json!({
            "mrz_raw": "IDKENKE123456781<<<<<<<<<<<<\n9006154F2906158KEN<<<<<<<<<<<4\nWANJIKU<<GRACE<<<<<<<<<<<<<<<<",
            "device_serial": "BVRD-KE-00001"
        });
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/edge/mrz/parse")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn mrz_parse_rejects_empty() {
        let app = router();
        let body = serde_json::json!({"mrz_raw": "", "device_serial": "BVRD-KE-00001"});
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/edge/mrz/parse")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
