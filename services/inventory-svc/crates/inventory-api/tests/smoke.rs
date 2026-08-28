use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use inventory_api::auth::Auth;
use inventory_api::router::{build_authed_router, build_router};
use inventory_api::store::Store;

fn app() -> axum::Router {
    build_router(Store::default())
}

async fn send(
    app: &axum::Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    let req = match body {
        Some(b) => {
            builder = builder.header("content-type", "application/json");
            builder.body(Body::from(b.to_string())).unwrap()
        }
        None => builder.body(Body::empty()).unwrap(),
    };
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, value)
}

#[tokio::test]
async fn health_returns_200() {
    let (status, _) = send(&app(), "GET", "/health", None).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn authed_router_rejects_missing_token() {
    let auth = Auth::from_endpoints("http://127.0.0.1:9/jwks", "http://127.0.0.1:9/spicedb");
    let app = build_authed_router(Store::default(), auth);

    // Protected route without a token -> 401 (short-circuits before any network).
    let (status, _) = send(&app, "GET", "/v1/transport/vehicles", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Health remains public.
    let (status, _) = send(&app, "GET", "/health", None).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn transport_load_scan_dispatch_deliver() {
    let app = app();

    // Register a vehicle.
    let (status, vehicle) = send(
        &app,
        "POST",
        "/v1/transport/vehicles",
        Some(json!({"registration": "KDE 123A", "vehicle_type": "truck", "capacity": 10})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let vehicle_id = vehicle["id"].as_str().unwrap().to_string();

    // Create a load.
    let (status, load) = send(
        &app,
        "POST",
        "/v1/transport/loads",
        Some(json!({"vehicle_id": vehicle_id, "destination": "KE-022"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let load_id = load["id"].as_str().unwrap().to_string();

    // Scan (RFID) and load a ballot lot.
    let (status, load) = send(
        &app,
        "POST",
        format!("/v1/transport/loads/{load_id}/items").as_str(),
        Some(json!({
            "lot_id": "LOT-1",
            "quantity": 6,
            "scan_method": "rfid",
            "scan_value": "EPC-3008-B94C-0001",
            "scanned_by": "officer-9"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(load["items"].as_array().unwrap().len(), 1);

    // Over capacity -> 422.
    let (status, _) = send(
        &app,
        "POST",
        format!("/v1/transport/loads/{load_id}/items").as_str(),
        Some(json!({
            "lot_id": "LOT-2", "quantity": 6, "scan_method": "barcode",
            "scan_value": "8901234", "scanned_by": "officer-9"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Dispatch then deliver.
    let (status, _) = send(
        &app,
        "POST",
        format!("/v1/transport/loads/{load_id}/dispatch").as_str(),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, load) = send(
        &app,
        "POST",
        format!("/v1/transport/loads/{load_id}/deliver").as_str(),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(load["status"], "delivered");
}

#[tokio::test]
async fn warehouse_transfer_and_pallet_lifecycle() {
    let app = app();

    let (_, a) = send(
        &app,
        "POST",
        "/v1/warehouse/warehouses",
        Some(json!({"code": "WH-M", "name": "Master", "tier": "master", "latitude": -1.29, "longitude": 36.82, "geofence_radius_m": 500.0})),
    )
    .await;
    let a_id = a["id"].as_str().unwrap().to_string();
    let (_, b) = send(
        &app,
        "POST",
        "/v1/warehouse/warehouses",
        Some(json!({"code": "WH-C", "name": "County", "tier": "county", "latitude": -1.30, "longitude": 36.83, "geofence_radius_m": 500.0})),
    )
    .await;
    let b_id = b["id"].as_str().unwrap().to_string();

    // Transfer with departure + arrival reconciliation.
    let (status, transfer) = send(
        &app,
        "POST",
        "/v1/warehouse/transfers",
        Some(json!({"from_warehouse_id": a_id, "to_warehouse_id": b_id, "expected_pallets": ["P-1", "P-2"]})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let transfer_id = transfer["id"].as_str().unwrap().to_string();

    let (status, _) = send(
        &app,
        "POST",
        format!("/v1/warehouse/transfers/{transfer_id}/depart").as_str(),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, arrived) = send(
        &app,
        "POST",
        format!("/v1/warehouse/transfers/{transfer_id}/arrive").as_str(),
        Some(json!({"arrived_pallets": ["P-1"]})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(arrived["reconciliation"]["missing_pallets"], json!(["P-2"]));

    // Pallet from receipt to use at the polling station.
    let (status, pallet) = send(
        &app,
        "POST",
        "/v1/warehouse/pallets",
        Some(json!({"serial": "KE27-PAL-0001", "warehouse_id": a_id})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let pallet_id = pallet["id"].as_str().unwrap().to_string();

    for (path, body) in [
        (format!("/v1/warehouse/pallets/{pallet_id}/store"), None),
        (
            format!("/v1/warehouse/pallets/{pallet_id}/break-bulk"),
            Some(
                json!({"items": [{"item_type": "ballot_box", "reference_id": "BOX-001", "quantity": 20}]}),
            ),
        ),
        (format!("/v1/warehouse/pallets/{pallet_id}/dispatch"), None),
        (
            format!("/v1/warehouse/pallets/{pallet_id}/used"),
            Some(json!({"station_code": "STN-022-004"})),
        ),
    ] {
        let (status, _) = send(&app, "POST", path.as_str(), body).await;
        assert_eq!(status, StatusCode::OK, "step {path}");
    }

    let (status, pallet) = send(
        &app,
        "GET",
        format!("/v1/warehouse/pallets/{pallet_id}").as_str(),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(pallet["status"], "used");
    assert_eq!(pallet["used_at_station"], "STN-022-004");
}

#[tokio::test]
async fn incident_create_and_resolve() {
    let app = app();
    let (status, incident) = send(
        &app,
        "POST",
        "/v1/warehouse/incidents",
        Some(json!({
            "incident_type": "tamper",
            "severity": "P0",
            "summary": "Seal broken outside custody event",
            "reported_by": "officer-9"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(incident["status"], "open");
    let incident_id = incident["id"].as_str().unwrap().to_string();

    let (status, resolved) = send(
        &app,
        "POST",
        format!("/v1/warehouse/incidents/{incident_id}/resolve").as_str(),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resolved["status"], "resolved");

    // Double-resolve -> conflict.
    let (status, _) = send(
        &app,
        "POST",
        format!("/v1/warehouse/incidents/{incident_id}/resolve").as_str(),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn custody_asset_and_event_chain() {
    let app = app();
    let (status, asset) = send(
        &app,
        "POST",
        "/v1/inventory/assets",
        Some(json!({"asset_type": "ballot", "serial": "KE27-BAL-001", "model": "BAL-2027"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let asset_id = asset["asset_id"].as_str().unwrap().to_string();

    let (status, event) = send(
        &app,
        "POST",
        "/v1/inventory/custody-events",
        Some(json!({
            "asset_id": asset_id,
            "from_custodian": "warehouse",
            "to_custodian": "transporter",
            "location": "WH-M",
            "signature": "sig"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(event["seq"], 1);

    let (status, history) = send(
        &app,
        "GET",
        format!("/v1/inventory/assets/{asset_id}/history").as_str(),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(history["events"].as_array().unwrap().len(), 1);
}
