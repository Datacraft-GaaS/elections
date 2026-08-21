//! IPsec tunnel health probe: shells out to `swanctl --list-sas` and exposes
//! GET /vpn/tunnels (JSON) and Prometheus metrics.
//!
//! Carriers: safaricom | airtel | telkom

use axum::{routing::get, Router};
use serde::Serialize;
use std::process::Command;
use tracing::info;

#[derive(Debug, Clone, Serialize)]
pub struct TunnelStatus {
    pub carrier: String,
    pub name: String,
    pub up: bool,
    pub bytes_in: u64,
    pub bytes_out: u64,
}

fn parse_swanctl_output(output: &str) -> Vec<TunnelStatus> {
    // Minimal parser: each CHILD_SA line contains "INSTALLED" when up.
    let carriers = ["safaricom", "airtel", "telkom"];
    let mut result = Vec::new();
    for carrier in &carriers {
        let up = output.contains(carrier) && output.contains("INSTALLED");
        result.push(TunnelStatus {
            carrier: carrier.to_string(),
            name: format!("eel-{carrier}"),
            up,
            bytes_in: 0,
            bytes_out: 0,
        });
    }
    result
}

async fn list_tunnels() -> axum::Json<Vec<TunnelStatus>> {
    let output = Command::new("swanctl")
        .args(["--list-sas", "--raw"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    axum::Json(parse_swanctl_output(&output))
}

async fn metrics() -> String {
    let output = Command::new("swanctl")
        .args(["--list-sas", "--raw"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    let tunnels = parse_swanctl_output(&output);
    tunnels
        .iter()
        .map(|t| {
            format!(
                "eel_vpn_tunnel_up{{carrier=\"{}\"}} {}",
                t.carrier,
                if t.up { 1 } else { 0 }
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({"status": "ok"}))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let addr = "0.0.0.0:8080";
    info!("eel-adapter-telco-vpn listening on {}", addr);
    let app = Router::new()
        .route("/vpn/tunnels", get(list_tunnels))
        .route("/metrics", get(metrics))
        .route("/health", get(health));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
