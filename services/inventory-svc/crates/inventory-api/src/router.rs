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

//! Axum router for inventory-svc.

use axum::http::StatusCode;
use axum::middleware;
use axum::routing::{get, post};
use axum::Router;

use crate::auth::{require_auth, Auth};
use crate::handlers::{custody, transport, warehouse};
use crate::spec;
use crate::store::Store;

/// Unauthenticated router (local development + hermetic tests).
pub fn build_router(store: Store) -> Router {
    Router::new()
        .route("/health", get(health))
        .merge(protected_routes(store))
}

/// Router with JWT + SpiceDB authorization on every protected route.
pub fn build_authed_router(store: Store, auth: Auth) -> Router {
    Router::new()
        .route("/health", get(health))
        .merge(protected_routes(store).layer(middleware::from_fn_with_state(auth, require_auth)))
}

fn protected_routes(store: Store) -> Router {
    Router::new()
        // Custody (M20)
        .route(
            "/v1/inventory/assets",
            get(custody::list_assets).post(custody::create_asset),
        )
        .route(
            "/v1/inventory/assets/{id}/history",
            get(custody::asset_history),
        )
        .route(
            "/v1/inventory/custody-events",
            post(custody::create_custody_event),
        )
        // Public contract endpoints (inventory/management.yaml)
        .route("/api/v1/inventory/kits", get(spec::list_kit_allocation_summary))
        .route("/api/v1/inventory/kits/{constituency_code}/allocation", get(spec::get_constituency_kit_allocation))
        .route("/api/v1/inventory/materials/{constituency_code}/status", get(spec::get_material_status))
        .route("/api/v1/inventory/shortfalls", get(spec::list_shortfalls).post(spec::report_shortfall))
        .route("/api/v1/inventory/audit-trail", get(spec::get_inventory_audit_trail))
        // Transport & security
        .route(
            "/v1/transport/vehicles",
            get(transport::list_vehicles).post(transport::create_vehicle),
        )
        .route("/v1/transport/loads", post(transport::create_load))
        .route(
            "/v1/transport/loads/{id}/items",
            post(transport::add_load_item),
        )
        .route(
            "/v1/transport/loads/{id}/dispatch",
            post(transport::dispatch_load),
        )
        .route(
            "/v1/transport/loads/{id}/deliver",
            post(transport::deliver_load),
        )
        .route(
            "/v1/transport/security-assignments",
            get(transport::list_security_assignments).post(transport::create_security_assignment),
        )
        // Warehouses, transfers, pallets, incidents
        .route(
            "/v1/warehouse/warehouses",
            get(warehouse::list_warehouses).post(warehouse::create_warehouse),
        )
        .route("/v1/warehouse/transfers", post(warehouse::create_transfer))
        .route("/v1/warehouse/transfers/{id}", get(warehouse::get_transfer))
        .route(
            "/v1/warehouse/transfers/{id}/depart",
            post(warehouse::depart_transfer),
        )
        .route(
            "/v1/warehouse/transfers/{id}/arrive",
            post(warehouse::arrive_transfer),
        )
        .route("/v1/warehouse/pallets", post(warehouse::receive_pallet))
        .route("/v1/warehouse/pallets/{id}", get(warehouse::get_pallet))
        .route(
            "/v1/warehouse/pallets/{id}/store",
            post(warehouse::store_pallet),
        )
        .route(
            "/v1/warehouse/pallets/{id}/break-bulk",
            post(warehouse::break_bulk_pallet),
        )
        .route(
            "/v1/warehouse/pallets/{id}/dispatch",
            post(warehouse::dispatch_pallet),
        )
        .route(
            "/v1/warehouse/pallets/{id}/used",
            post(warehouse::mark_pallet_used),
        )
        .route(
            "/v1/warehouse/incidents",
            get(warehouse::list_incidents).post(warehouse::create_incident),
        )
        .route(
            "/v1/warehouse/incidents/{id}/resolve",
            post(warehouse::resolve_incident),
        )
        .with_state(store)
}

async fn health() -> StatusCode {
    StatusCode::OK
}
