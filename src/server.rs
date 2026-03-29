// Unlicense — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.6

//! Core web server — serves user's site from phone storage,
//! kiosk dashboard for the phone screen, stats API.

use axum::{
    extract::{ConnectInfo, Multipart, Request, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;

use crate::govdocs;
use crate::pwa;
use crate::stats::t1;

/// t0=AppState
#[allow(non_camel_case_types)]
pub struct t0 {
    /// s0=stats
    pub s0: Arc<t1>,
    /// s1=site_name
    pub s1: String,
    /// s2=hostname
    pub s2: String,
    /// s3=site_dir — directory on phone storage containing user's site files
    pub s3: Option<PathBuf>,
}

/// f0=dashboard_page — kiosk dashboard, the phone screen shows this
fn f0(state: &t0) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1,user-scalable=no">
<title>{name} — Dashboard</title>
{pwa}
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
if('serviceWorker' in navigator)navigator.serviceWorker.register('/sw.js');
</script>
</body>
</html>"##,
        name = state.s1,
        pwa = pwa::PWA_HEAD,
    )
}

/// f1=landing_page — shown to visitors when no user site is loaded
fn f1(state: &t0) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>{name}</title>
{pwa}
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
        name = state.s1,
        pwa = pwa::PWA_HEAD,
    )
}

/// f2=dashboard — handler
async fn f2(State(state): State<Arc<t0>>) -> impl IntoResponse {
    Html(f0(&state))
}

/// f3=fallback_index
async fn f3(State(state): State<Arc<t0>>) -> impl IntoResponse {
    let body = f1(&state);
    let len = body.len() as u64;
    state.s0.f11(len);
    Html(body)
}

/// f4=api_stats
async fn f4(State(state): State<Arc<t0>>) -> impl IntoResponse {
    let json = state.s0.f19();
    state.s0.f11(json.len() as u64);
    (
        StatusCode::OK,
        [("content-type", "application/json")],
        json,
    )
}

/// f5=health
async fn f5() -> &'static str {
    "OK"
}

/// f6=upload — upload files to site directory, localhost only
async fn f6(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<t0>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    if !addr.ip().is_loopback() {
        return (StatusCode::FORBIDDEN, "upload restricted to localhost".to_string());
    }
    let base = match &state.s3 {
        Some(d) => d.clone(),
        None => return (StatusCode::BAD_REQUEST, "no site directory configured".to_string()),
    };

    let mut count = 0u32;
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = match field.file_name() {
            Some(n) => n.to_string(),
            None => continue,
        };

        // Sanitize: normalize separators, strip leading /, reject .. components
        let normalized = file_name.replace('\\', "/");
        let clean: PathBuf = normalized
            .split('/')
            .filter(|c| !c.is_empty() && *c != "." && *c != "..")
            .collect();
        if clean.as_os_str().is_empty() {
            continue;
        }
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

/// f7=counting_layer — middleware that counts bytes for static file responses
async fn f7(
    State(state): State<Arc<t0>>,
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
        state.s0.f11(size);
    }
    resp
}

/// f8=build_router
pub fn f8(state: t0) -> Router {
    let shared = Arc::new(state);

    let mut app = Router::new()
        .route("/dashboard", get(f2))
        .route("/api/stats", get(f4))
        .route("/api/upload", post(f6))
        .route("/health", get(f5))
        .route("/govdocs", get(govdocs::f23))
        .route("/govdocs/sbom", get(govdocs::f24))
        .route("/govdocs/capability", get(govdocs::f25))
        .route("/govdocs/security", get(govdocs::f26))
        .route("/manifest.json", get(pwa::f27))
        .route("/sw.js", get(pwa::f28))
        .route("/pwa/icon.svg", get(pwa::f29));

    if let Some(ref dir) = shared.s3 {
        let serve = ServeDir::new(dir).append_index_html_on_directories(true);
        app = app
            .nest_service("/", serve)
            .layer(axum::middleware::from_fn_with_state(
                shared.clone(),
                f7,
            ));
    } else {
        app = app.route("/", get(f3));
    }

    app.layer(CompressionLayer::new().zstd(true))
        .with_state(shared)
}

/// f9=run — start the server on the given port, blocking
pub async fn f9(s1: String, s2: String, port: u16, s3: Option<PathBuf>) {
    let state = t0 {
        s0: Arc::new(t1::f10()),
        s1,
        s2,
        s3,
    };
    let app = f8(state);
    let addr = format!("0.0.0.0:{}", port);
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => {
            eprintln!("  listening on {}", addr);
            l
        }
        Err(e) => {
            eprintln!("error: cannot bind to {} — {}", addr, e);
            std::process::exit(1);
        }
    };
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
