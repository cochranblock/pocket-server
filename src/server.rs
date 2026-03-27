// Unlicense — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.6

//! Core web server — serves user's site from phone storage,
//! kiosk dashboard for the phone screen, stats API.

use axum::{
    extract::{Multipart, Request, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;

use crate::stats::Stats;

pub struct AppState {
    pub stats: Arc<Stats>,
    pub site_name: String,
    pub hostname: String,
    /// Directory on phone storage containing user's site files.
    /// None = serve default landing page at /.
    pub site_dir: Option<PathBuf>,
}

// ---------------------------------------------------------------------------
// Kiosk dashboard — the phone screen shows this
// ---------------------------------------------------------------------------

fn dashboard_page(state: &AppState) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1,user-scalable=no">
<title>{name} — Dashboard</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
html,body{{height:100%;overflow:hidden}}
body{{background:#0a0a0a;color:#e0e0e0;font-family:'SF Mono',ui-monospace,monospace;display:flex;flex-direction:column;align-items:center;justify-content:center;min-height:100vh}}
.hdr{{font-size:1.1rem;color:#00d4aa;letter-spacing:0.15em;text-transform:uppercase;margin-bottom:2rem}}
.grid{{display:grid;grid-template-columns:1fr 1fr;gap:1.5rem 3rem;text-align:center}}
.stat{{display:flex;flex-direction:column;align-items:center}}
.val{{font-size:3rem;font-weight:700;color:#fff;line-height:1}}
.lbl{{font-size:0.75rem;color:#555;text-transform:uppercase;letter-spacing:0.1em;margin-top:0.3rem}}
.wide{{grid-column:1/-1}}
.pulse{{width:8px;height:8px;border-radius:50%;background:#00d4aa;display:inline-block;margin-right:0.5rem;animation:blink 2s infinite}}
@keyframes blink{{0%,100%{{opacity:1}}50%{{opacity:0.3}}}}
.foot{{position:fixed;bottom:1rem;font-size:0.65rem;color:#333}}
</style>
</head>
<body>
<div class="hdr"><span class="pulse"></span>{name}</div>
<div class="grid">
  <div class="stat"><span class="val" id="up">--</span><span class="lbl">Uptime</span></div>
  <div class="stat"><span class="val" id="req">--</span><span class="lbl">Requests</span></div>
  <div class="stat"><span class="val" id="bw">--</span><span class="lbl">Served</span></div>
  <div class="stat"><span class="val" id="pw">--</span><span class="lbl">Watts</span></div>
  <div class="stat wide"><span class="val" id="cost">--</span><span class="lbl">Est. Monthly Cost</span></div>
</div>
<div class="foot">Pocket Server · cochranblock.org</div>
<script>
async function poll(){{
  try{{
    const r=await fetch('/api/stats');
    const d=await r.json();
    document.getElementById('up').textContent=d.uptime;
    document.getElementById('req').textContent=d.requests.toLocaleString();
    document.getElementById('bw').textContent=d.bytes_served;
    document.getElementById('pw').textContent=d.power_w.toFixed(1)+'W';
    document.getElementById('cost').textContent=d.monthly_cost;
  }}catch(_){{}}
}}
poll();
setInterval(poll,2000);
</script>
</body>
</html>"##,
        name = state.site_name
    )
}

// ---------------------------------------------------------------------------
// Default landing page — shown to visitors when no user site is loaded
// ---------------------------------------------------------------------------

fn landing_page(state: &AppState) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>{name}</title>
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
<h1>{name}</h1>
<p>This site is served from a phone.</p>
<p class="accent">No cloud. No hosting bill. Ever.</p>
<p style="margin-top:2rem;font-size:0.8rem;color:#555">Powered by Pocket Server · cochranblock.org</p>
</div>
</body>
</html>"#,
        name = state.site_name
    )
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn dashboard(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Html(dashboard_page(&state))
}

async fn fallback_index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let body = landing_page(&state);
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

/// Upload files to the site directory.
/// POST /api/upload with multipart/form-data — field name "file", optional "path" for subdirs.
async fn upload(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let base = match &state.site_dir {
        Some(d) => d.clone(),
        None => return (StatusCode::BAD_REQUEST, "no site directory configured".to_string()),
    };

    let mut count = 0u32;
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = match field.file_name() {
            Some(n) => n.to_string(),
            None => continue,
        };

        // Sanitize: no path traversal
        let clean = file_name.replace("..", "").replace('\\', "/");
        let dest = base.join(&clean);

        if let Some(parent) = dest.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }

        match field.bytes().await {
            Ok(data) => {
                if let Err(e) = tokio::fs::write(&dest, &data).await {
                    return (StatusCode::INTERNAL_SERVER_ERROR, format!("write error: {}", e));
                }
                count += 1;
            }
            Err(e) => {
                return (StatusCode::BAD_REQUEST, format!("read error: {}", e));
            }
        }
    }

    (StatusCode::OK, format!("{} file(s) uploaded", count))
}

/// Middleware that counts bytes for static file responses.
async fn counting_layer(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: axum::middleware::Next,
) -> Response {
    let resp = next.run(req).await;
    if resp.status().is_success() {
        let size = resp
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);
        state.stats.record_request(size);
    }
    resp
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// Build the router. Caller provides AppState with site config.
pub fn build_router(state: AppState) -> Router {
    let shared = Arc::new(state);

    let mut app = Router::new()
        .route("/dashboard", get(dashboard))
        .route("/api/stats", get(api_stats))
        .route("/api/upload", post(upload))
        .route("/health", get(health));

    // If a site directory is configured, serve it at /. Otherwise landing page.
    if let Some(ref dir) = shared.site_dir {
        let serve = ServeDir::new(dir).append_index_html_on_directories(true);
        app = app
            .nest_service("/", serve)
            .layer(axum::middleware::from_fn_with_state(
                shared.clone(),
                counting_layer,
            ));
    } else {
        app = app.route("/", get(fallback_index));
    }

    app.layer(CompressionLayer::new().zstd(true))
        .with_state(shared)
}

/// Start the server on the given port. Blocking.
pub async fn run(site_name: String, hostname: String, port: u16, site_dir: Option<PathBuf>) {
    let state = AppState {
        stats: Arc::new(Stats::new()),
        site_name,
        hostname,
        site_dir,
    };
    let app = build_router(state);
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
