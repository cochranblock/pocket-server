// Unlicense — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.6

//! Core web server — serves user's site content from embedded or loaded assets.
//! Tracks every request through Stats for the dashboard.

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::sync::Arc;
use tower_http::compression::CompressionLayer;

use crate::stats::Stats;

pub struct AppState {
    pub stats: Stats,
    pub site_name: String,
    pub hostname: String,
}

/// Default landing page when no custom site is loaded.
fn default_page(state: &AppState) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>{}</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{background:#0a0a0a;color:#e0e0e0;font-family:system-ui;display:flex;align-items:center;justify-content:center;min-height:100vh}}
.card{{text-align:center;padding:3rem}}
h1{{font-size:2.5rem;color:#00d4aa;margin-bottom:1rem}}
p{{font-size:1.2rem;color:#888;margin-bottom:0.5rem}}
.accent{{color:#00d4aa}}
</style>
</head>
<body>
<div class="card">
<h1>{}</h1>
<p>This site is served from a phone.</p>
<p class="accent">No cloud. No hosting bill. Ever.</p>
<p style="margin-top:2rem;font-size:0.8rem;color:#555">Powered by Pocket Server · cochranblock.org</p>
</div>
</body>
</html>"#,
        state.site_name, state.site_name
    )
}

async fn index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let body = default_page(&state);
    let len = body.len() as u64;
    state.stats.record_request(len);
    Html(body)
}

async fn api_stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let json = state.stats.to_json();
    state.stats.record_request(json.len() as u64);
    (
        StatusCode::OK,
        [("content-type", "application/json")],
        json,
    )
}

async fn health() -> &'static str {
    "OK"
}

/// Build the router. Caller provides AppState with site config.
pub fn build_router(state: AppState) -> Router {
    let state = Arc::new(state);
    Router::new()
        .route("/", get(index))
        .route("/api/stats", get(api_stats))
        .route("/health", get(health))
        .layer(CompressionLayer::new().zstd(true))
        .with_state(state)
}

/// Start the server on the given port. Blocking.
pub async fn run(site_name: String, hostname: String, port: u16) {
    let state = AppState {
        stats: Stats::new(),
        site_name,
        hostname,
    };
    let app = build_router(state);
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
