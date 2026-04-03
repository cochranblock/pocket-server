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
    /// s7=quiet — suppress access log output
    pub s7: bool,
    /// s8=max_upload — max upload size in bytes (0 = unlimited)
    pub s8: u64,
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
                if state.s8 > 0 && data.len() as u64 > state.s8 {
                    return (
                        StatusCode::PAYLOAD_TOO_LARGE,
                        format!("file too large: {} bytes (max {})", data.len(), state.s8),
                    );
                }
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

/// f30=access_log — middleware that logs method, path, status, duration to stderr
async fn f30(
    State(state): State<Arc<t0>>,
    req: Request,
    next: axum::middleware::Next,
) -> Response {
    if state.s7 {
        return next.run(req).await;
    }
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let start = std::time::Instant::now();
    let resp = next.run(req).await;
    let elapsed = start.elapsed();
    eprintln!(
        "  {} {} {} {:.1}ms",
        method,
        path,
        resp.status().as_u16(),
        elapsed.as_secs_f64() * 1000.0
    );
    resp
}

/// f31=list_files — GET /api/files — list files in site directory
async fn f31(State(state): State<Arc<t0>>) -> impl IntoResponse {
    let base = match &state.s3 {
        Some(d) => d.clone(),
        None => return (StatusCode::BAD_REQUEST, "no site directory configured".to_string()),
    };
    let mut entries = Vec::new();
    if let Ok(stack) = tokio::fs::read_dir(&base).await.map(|r| vec![r]) {
        // Iterative directory walk
        let mut dirs = vec![base.clone()];
        entries.clear();
        while let Some(dir) = dirs.pop() {
            let mut rd = match tokio::fs::read_dir(&dir).await {
                Ok(rd) => rd,
                Err(_) => continue,
            };
            while let Ok(Some(entry)) = rd.next_entry().await {
                let path = entry.path();
                if path.is_dir() {
                    dirs.push(path);
                } else if let Ok(meta) = entry.metadata().await {
                    let rel = path.strip_prefix(&base).unwrap_or(&path);
                    entries.push(format!(
                        r#"{{"path":"{}","size":{}}}"#,
                        rel.display().to_string().replace('\\', "/"),
                        meta.len()
                    ));
                }
            }
        }
        drop(stack);
    }
    let json = format!("[{}]", entries.join(","));
    (StatusCode::OK, json)
}

/// f32=delete_file — DELETE /api/files/*path — delete a file (localhost only)
async fn f32(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<t0>>,
    axum::extract::Path(file_path): axum::extract::Path<String>,
) -> impl IntoResponse {
    if !addr.ip().is_loopback() {
        return (StatusCode::FORBIDDEN, "delete restricted to localhost".to_string());
    }
    let base = match &state.s3 {
        Some(d) => d.clone(),
        None => return (StatusCode::BAD_REQUEST, "no site directory configured".to_string()),
    };
    let path = file_path.as_str();
    if path.is_empty() {
        return (StatusCode::BAD_REQUEST, "no file path specified".to_string());
    }
    // Sanitize same as upload
    let normalized = path.replace('\\', "/");
    let clean: PathBuf = normalized
        .split('/')
        .filter(|c| !c.is_empty() && *c != "." && *c != "..")
        .collect();
    if clean.as_os_str().is_empty() {
        return (StatusCode::BAD_REQUEST, "invalid path".to_string());
    }
    let target = base.join(&clean);
    if !target.exists() {
        return (StatusCode::NOT_FOUND, "file not found".to_string());
    }
    match tokio::fs::remove_file(&target).await {
        Ok(_) => (StatusCode::OK, format!("deleted {}", clean.display())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("delete error: {}", e)),
    }
}

/// f8=build_router
pub fn f8(state: t0) -> Router {
    let shared = Arc::new(state);

    let mut app = Router::new()
        .route("/dashboard", get(f2))
        .route("/api/stats", get(f4))
        .route("/api/upload", post(f6))
        .route("/health", get(f5))
        .route("/api/files", get(f31))
        .route("/api/files/*path", axum::routing::delete(f32))
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

    app.layer(axum::middleware::from_fn_with_state(shared.clone(), f30))
        .layer(CompressionLayer::new().zstd(true))
        .with_state(shared)
}

/// f9=run — start the server on the given port, blocking. Shuts down on SIGINT/SIGTERM.
pub async fn f9(s1: String, s2: String, port: u16, s3: Option<PathBuf>, quiet: bool, max_upload: u64) {
    let state = t0 {
        s0: Arc::new(t1::f10()),
        s1,
        s2,
        s3,
        s7: quiet,
        s8: max_upload,
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
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();
    eprintln!("  server stopped");
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();
    #[cfg(unix)]
    {
        let mut sigterm =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
        tokio::select! {
            _ = ctrl_c => eprintln!("\n  shutting down (SIGINT)..."),
            _ = sigterm.recv() => eprintln!("\n  shutting down (SIGTERM)..."),
        }
    }
    #[cfg(not(unix))]
    {
        ctrl_c.await.ok();
        eprintln!("\n  shutting down...");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn test_state(with_dir: bool) -> t0 {
        t0 {
            s0: Arc::new(t1::f10()),
            s1: "Test Server".to_string(),
            s2: "test-host".to_string(),
            s3: if with_dir {
                let dir = std::env::temp_dir().join("pocket-server-test-site");
                let _ = std::fs::create_dir_all(&dir);
                Some(dir)
            } else {
                None
            },
            s7: true, // quiet in tests
            s8: 0,    // unlimited in tests
        }
    }

    fn req(method: &str, uri: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/health")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"OK");
    }

    #[tokio::test]
    async fn dashboard_returns_html() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/dashboard")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        )
        .unwrap();
        assert!(body.contains("Test Server"));
        assert!(body.contains("cochranblock.org"));
        assert!(body.contains("/api/stats"));
    }

    #[tokio::test]
    async fn api_stats_returns_json() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/api/stats")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
        assert!(ct.contains("application/json"));
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        )
        .unwrap();
        assert!(body.contains("\"uptime\""));
        assert!(body.contains("\"requests\""));
        assert!(body.contains("\"power_w\""));
    }

    #[tokio::test]
    async fn fallback_landing_when_no_site_dir() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        )
        .unwrap();
        assert!(body.contains("No cloud. No hosting bill. Ever."));
    }

    #[tokio::test]
    async fn govdocs_index_returns_html() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/govdocs")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        )
        .unwrap();
        assert!(body.contains("Compliance Documents"));
    }

    #[tokio::test]
    async fn govdocs_sbom_returns_html() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/govdocs/sbom")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        )
        .unwrap();
        assert!(body.contains("Software Bill of Materials"));
    }

    #[tokio::test]
    async fn govdocs_sbom_spdx_format() {
        let app = f8(test_state(false));
        let resp = app
            .oneshot(req("GET", "/govdocs/sbom?format=spdx"))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        )
        .unwrap();
        assert!(body.starts_with("SPDXVersion: SPDX-2.3"));
        assert!(body.contains("PackageName: pocket-server"));
    }

    #[tokio::test]
    async fn govdocs_capability_returns_html() {
        let app = f8(test_state(false));
        let resp = app
            .oneshot(req("GET", "/govdocs/capability"))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        )
        .unwrap();
        assert!(body.contains("Capability"));
        assert!(body.contains("cochranblock.org"));
    }

    #[tokio::test]
    async fn govdocs_security_returns_html() {
        let app = f8(test_state(false));
        let resp = app
            .oneshot(req("GET", "/govdocs/security"))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        )
        .unwrap();
        assert!(body.contains("Security"));
    }

    #[tokio::test]
    async fn pwa_manifest_returns_json() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/manifest.json")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        )
        .unwrap();
        assert!(body.contains("\"name\""));
        assert!(body.contains("\"start_url\""));
    }

    #[tokio::test]
    async fn pwa_service_worker() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/sw.js")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        )
        .unwrap();
        assert!(body.contains("install"));
        assert!(body.contains("fetch"));
    }

    #[tokio::test]
    async fn pwa_icon_returns_svg() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/pwa/icon.svg")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        )
        .unwrap();
        assert!(body.contains("<svg"));
    }

    #[tokio::test]
    async fn api_files_list_empty_dir() {
        let app = f8(test_state(true));
        let resp = app.oneshot(req("GET", "/api/files")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        )
        .unwrap();
        assert!(body.starts_with('['));
        assert!(body.ends_with(']'));
    }

    #[tokio::test]
    async fn api_files_no_site_dir() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/api/files")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn upload_requires_connect_info() {
        // Without ConnectInfo layer, upload returns 400 (extractor rejection)
        // With proper ConnectInfo + non-loopback IP, handler returns 403
        // This test verifies the endpoint exists and rejects bare requests
        let app = f8(test_state(true));
        let req = Request::builder()
            .method("POST")
            .uri("/api/upload")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        // 500 = ConnectInfo extractor missing (expected without make_service layer)
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn dashboard_contains_pwa_meta() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/dashboard")).await.unwrap();
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        ).unwrap();
        assert!(body.contains("manifest.json"));
        assert!(body.contains("theme-color"));
        assert!(body.contains("apple-mobile-web-app-capable"));
    }

    #[tokio::test]
    async fn dashboard_contains_js_polling() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/dashboard")).await.unwrap();
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        ).unwrap();
        assert!(body.contains("setInterval(poll,2000)"));
        assert!(body.contains("fetch('/api/stats')"));
    }

    #[tokio::test]
    async fn dashboard_contains_css() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/dashboard")).await.unwrap();
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        ).unwrap();
        assert!(body.contains("#0a0a0a"));
        assert!(body.contains("#00d4aa"));
    }

    #[tokio::test]
    async fn dashboard_has_stat_elements() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/dashboard")).await.unwrap();
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        ).unwrap();
        assert!(body.contains("id=\"up\""));
        assert!(body.contains("id=\"req\""));
        assert!(body.contains("id=\"bw\""));
        assert!(body.contains("id=\"pw\""));
        assert!(body.contains("id=\"cost\""));
    }

    #[tokio::test]
    async fn landing_page_contains_pwa_meta() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/")).await.unwrap();
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        ).unwrap();
        assert!(body.contains("manifest.json"));
    }

    #[tokio::test]
    async fn landing_page_site_name() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/")).await.unwrap();
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        ).unwrap();
        assert!(body.contains("Test Server"));
    }

    #[tokio::test]
    async fn fallback_counts_bytes() {
        let state = test_state(false);
        let stats = state.s0.clone();
        let app = f8(state);
        let _ = app.oneshot(req("GET", "/")).await.unwrap();
        assert!(stats.f14() > 0, "request not counted");
        assert!(stats.f15() > 0, "bytes not counted");
    }

    #[tokio::test]
    async fn api_stats_counts_bytes() {
        let state = test_state(false);
        let stats = state.s0.clone();
        let app = f8(state);
        let _ = app.oneshot(req("GET", "/api/stats")).await.unwrap();
        assert!(stats.f14() > 0);
        assert!(stats.f15() > 0);
    }

    #[tokio::test]
    async fn api_stats_json_structure() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/api/stats")).await.unwrap();
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        ).unwrap();
        // Verify all 5 JSON fields
        assert!(body.contains("\"uptime\":"));
        assert!(body.contains("\"requests\":"));
        assert!(body.contains("\"bytes_served\":"));
        assert!(body.contains("\"power_w\":"));
        assert!(body.contains("\"monthly_cost\":"));
        // Verify balanced braces
        assert!(body.starts_with('{'));
        assert!(body.ends_with('}'));
    }

    #[tokio::test]
    async fn health_exact_body() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/health")).await.unwrap();
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"OK");
    }

    #[tokio::test]
    async fn upload_no_site_dir() {
        let app = f8(test_state(false));
        let req = Request::builder()
            .method("POST")
            .uri("/api/upload")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        // Without ConnectInfo → 500 (extractor fail)
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn pwa_manifest_content_type() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/manifest.json")).await.unwrap();
        let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
        assert!(ct.contains("manifest+json") || ct.contains("application/json"));
    }

    #[tokio::test]
    async fn pwa_manifest_required_fields() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/manifest.json")).await.unwrap();
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        ).unwrap();
        assert!(body.contains("\"name\""));
        assert!(body.contains("\"short_name\""));
        assert!(body.contains("\"start_url\""));
        assert!(body.contains("\"display\""));
        assert!(body.contains("\"icons\""));
        assert!(body.contains("\"theme_color\""));
    }

    #[tokio::test]
    async fn pwa_sw_content_type() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/sw.js")).await.unwrap();
        let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
        assert!(ct.contains("javascript"));
    }

    #[tokio::test]
    async fn pwa_sw_has_cache_strategy() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/sw.js")).await.unwrap();
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        ).unwrap();
        assert!(body.contains("caches.open"));
        assert!(body.contains("pocket-server-v1"));
    }

    #[tokio::test]
    async fn pwa_icon_content_type() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/pwa/icon.svg")).await.unwrap();
        let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
        assert!(ct.contains("svg"));
    }

    #[tokio::test]
    async fn pwa_icon_valid_svg() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/pwa/icon.svg")).await.unwrap();
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        ).unwrap();
        assert!(body.contains("<svg"));
        assert!(body.contains("</svg>"));
        assert!(body.contains("viewBox"));
    }

    #[tokio::test]
    async fn govdocs_index_links() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/govdocs")).await.unwrap();
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        ).unwrap();
        assert!(body.contains("/govdocs/sbom"));
        assert!(body.contains("/govdocs/capability"));
        assert!(body.contains("/govdocs/security"));
    }

    #[tokio::test]
    async fn govdocs_index_version() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/govdocs")).await.unwrap();
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        ).unwrap();
        assert!(body.contains(env!("CARGO_PKG_VERSION")));
    }

    #[tokio::test]
    async fn govdocs_sbom_package_count() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/govdocs/sbom")).await.unwrap();
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        ).unwrap();
        // Should have a table with many rows
        let row_count = body.matches("<tr>").count();
        assert!(row_count > 10, "expected >10 package rows, got {}", row_count);
    }

    #[tokio::test]
    async fn govdocs_sbom_spdx_content_type() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/govdocs/sbom?format=spdx")).await.unwrap();
        let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
        assert!(ct.contains("text/plain"));
    }

    #[tokio::test]
    async fn delete_no_site_dir() {
        let app = f8(test_state(false));
        let mut req = Request::builder()
            .method("DELETE")
            .uri("/api/files/test.txt")
            .body(Body::empty())
            .unwrap();
        req.extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345))));
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn nonexistent_route_404() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/nonexistent")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn post_to_health_method_not_allowed() {
        let app = f8(test_state(false));
        let r = Request::builder()
            .method("POST")
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(r).await.unwrap();
        assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn get_to_upload_method_not_allowed() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/api/upload")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn upload_localhost_via_tcp() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let state = test_state(true);
        let app = f8(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await
            .unwrap();
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let mut stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port))
            .await
            .unwrap();
        let body = "------boundary--\r\n";
        let req = format!(
            "POST /api/upload HTTP/1.1\r\nHost: localhost\r\nContent-Type: multipart/form-data; boundary=----boundary\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(req.as_bytes()).await.unwrap();
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await.unwrap();
        let resp = String::from_utf8_lossy(&buf[..n]);
        // Localhost → 200 OK, "0 file(s) uploaded"
        assert!(resp.starts_with("HTTP/1.1 200"));
        assert!(resp.contains("0 file(s) uploaded"));
    }

    // Helper: start a real server and return (port, site_dir)
    async fn start_test_server() -> (u16, PathBuf) {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("ps-test-{}-{}", std::process::id(), id));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let state = t0 {
            s0: Arc::new(t1::f10()),
            s1: "Test".to_string(),
            s2: "test".to_string(),
            s3: Some(dir.clone()),
            s7: true,
            s8: 1024, // 1 KB limit for testing
        };
        let app = f8(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
                .await.unwrap();
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        (port, dir)
    }

    // Helper: raw HTTP request over TCP
    async fn tcp_request(port: u16, request: &str) -> String {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port))
            .await.unwrap();
        stream.write_all(request.as_bytes()).await.unwrap();
        let mut buf = vec![0u8; 65536];
        let n = stream.read(&mut buf).await.unwrap();
        String::from_utf8_lossy(&buf[..n]).to_string()
    }

    // Helper: multipart upload body
    fn multipart_body(filename: &str, content: &[u8]) -> Vec<u8> {
        let boundary = "----TestBoundary";
        let mut body = Vec::new();
        body.extend_from_slice(format!(
            "--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\nContent-Type: application/octet-stream\r\n\r\n",
            boundary, filename
        ).as_bytes());
        body.extend_from_slice(content);
        body.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());
        body
    }

    #[tokio::test]
    async fn upload_real_file() {
        let (port, dir) = start_test_server().await;
        let body = multipart_body("hello.txt", b"Hello, world!");
        let req = format!(
            "POST /api/upload HTTP/1.1\r\nHost: localhost\r\nContent-Type: multipart/form-data; boundary=----TestBoundary\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await.unwrap();
        stream.write_all(req.as_bytes()).await.unwrap();
        stream.write_all(&body).await.unwrap();
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await.unwrap();
        let resp = String::from_utf8_lossy(&buf[..n]);
        assert!(resp.contains("1 file(s) uploaded"), "resp: {}", resp);
        // Verify file on disk
        let content = std::fs::read_to_string(dir.join("hello.txt")).unwrap();
        assert_eq!(content, "Hello, world!");
    }

    #[tokio::test]
    async fn upload_subdirectory() {
        let (port, dir) = start_test_server().await;
        let body = multipart_body("sub/dir/test.txt", b"nested");
        let req = format!(
            "POST /api/upload HTTP/1.1\r\nHost: localhost\r\nContent-Type: multipart/form-data; boundary=----TestBoundary\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await.unwrap();
        stream.write_all(req.as_bytes()).await.unwrap();
        stream.write_all(&body).await.unwrap();
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await.unwrap();
        let resp = String::from_utf8_lossy(&buf[..n]);
        assert!(resp.contains("1 file(s) uploaded"), "resp: {}", resp);
        assert!(dir.join("sub/dir/test.txt").exists());
    }

    #[tokio::test]
    async fn upload_path_traversal_blocked() {
        let (port, dir) = start_test_server().await;
        let body = multipart_body("../../etc/passwd", b"hacked");
        let req = format!(
            "POST /api/upload HTTP/1.1\r\nHost: localhost\r\nContent-Type: multipart/form-data; boundary=----TestBoundary\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await.unwrap();
        stream.write_all(req.as_bytes()).await.unwrap();
        stream.write_all(&body).await.unwrap();
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await.unwrap();
        let resp = String::from_utf8_lossy(&buf[..n]);
        assert!(resp.contains("200"), "resp: {}", resp);
        // File should be inside site dir, not at /etc/passwd
        assert!(!std::path::Path::new("/tmp/etc/passwd").exists());
        // .. components stripped, so it lands in site_dir/etc/passwd
        assert!(dir.join("etc/passwd").exists());
    }

    #[tokio::test]
    async fn upload_backslash_traversal_blocked() {
        let (port, dir) = start_test_server().await;
        let body = multipart_body("..\\..\\test.txt", b"hacked");
        let req = format!(
            "POST /api/upload HTTP/1.1\r\nHost: localhost\r\nContent-Type: multipart/form-data; boundary=----TestBoundary\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await.unwrap();
        stream.write_all(req.as_bytes()).await.unwrap();
        stream.write_all(&body).await.unwrap();
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await.unwrap();
        let resp = String::from_utf8_lossy(&buf[..n]);
        assert!(resp.contains("200"), "resp: {}", resp);
        // Should end up in site_dir/test.txt
        assert!(dir.join("test.txt").exists());
    }

    #[tokio::test]
    async fn upload_dot_filename_skipped() {
        let (port, _dir) = start_test_server().await;
        let body = multipart_body(".", b"dot");
        let req = format!(
            "POST /api/upload HTTP/1.1\r\nHost: localhost\r\nContent-Type: multipart/form-data; boundary=----TestBoundary\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await.unwrap();
        stream.write_all(req.as_bytes()).await.unwrap();
        stream.write_all(&body).await.unwrap();
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await.unwrap();
        let resp = String::from_utf8_lossy(&buf[..n]);
        // Dot-only filename should be skipped
        assert!(resp.contains("0 file(s) uploaded"), "resp: {}", resp);
    }

    #[tokio::test]
    async fn upload_size_limit_rejected() {
        let (port, _dir) = start_test_server().await;
        // Server has 1024 byte limit, send 2048 bytes
        let big_content = vec![b'X'; 2048];
        let body = multipart_body("big.txt", &big_content);
        let req = format!(
            "POST /api/upload HTTP/1.1\r\nHost: localhost\r\nContent-Type: multipart/form-data; boundary=----TestBoundary\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await.unwrap();
        stream.write_all(req.as_bytes()).await.unwrap();
        stream.write_all(&body).await.unwrap();
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await.unwrap();
        let resp = String::from_utf8_lossy(&buf[..n]);
        assert!(resp.contains("413"), "expected 413, resp: {}", resp);
        assert!(resp.contains("file too large"), "resp: {}", resp);
    }

    #[tokio::test]
    async fn upload_within_size_limit_accepted() {
        let (port, dir) = start_test_server().await;
        // Server has 1024 byte limit, send 512 bytes
        let small_content = vec![b'Y'; 512];
        let body = multipart_body("small.txt", &small_content);
        let req = format!(
            "POST /api/upload HTTP/1.1\r\nHost: localhost\r\nContent-Type: multipart/form-data; boundary=----TestBoundary\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await.unwrap();
        stream.write_all(req.as_bytes()).await.unwrap();
        stream.write_all(&body).await.unwrap();
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await.unwrap();
        let resp = String::from_utf8_lossy(&buf[..n]);
        assert!(resp.contains("1 file(s) uploaded"), "resp: {}", resp);
        assert_eq!(std::fs::read(dir.join("small.txt")).unwrap().len(), 512);
    }

    #[tokio::test]
    async fn api_files_lists_uploaded_files() {
        let (port, dir) = start_test_server().await;
        // Create files directly
        std::fs::write(dir.join("a.txt"), "alpha").unwrap();
        std::fs::write(dir.join("b.txt"), "beta").unwrap();
        let resp = tcp_request(port, "GET /api/files HTTP/1.1\r\nHost: localhost\r\n\r\n").await;
        assert!(resp.contains("200"));
        assert!(resp.contains("a.txt"));
        assert!(resp.contains("b.txt"));
        assert!(resp.contains("\"size\":5")); // "alpha" = 5 bytes
        assert!(resp.contains("\"size\":4")); // "beta" = 4 bytes
    }

    #[tokio::test]
    async fn api_files_lists_subdirectories() {
        let (port, dir) = start_test_server().await;
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("sub/nested.txt"), "deep").unwrap();
        let resp = tcp_request(port, "GET /api/files HTTP/1.1\r\nHost: localhost\r\n\r\n").await;
        assert!(resp.contains("sub/nested.txt"));
    }

    #[tokio::test]
    async fn delete_file_success() {
        let (port, dir) = start_test_server().await;
        std::fs::write(dir.join("delete-me.txt"), "gone").unwrap();
        assert!(dir.join("delete-me.txt").exists());
        let resp = tcp_request(port, "DELETE /api/files/delete-me.txt HTTP/1.1\r\nHost: localhost\r\n\r\n").await;
        assert!(resp.contains("200"), "resp: {}", resp);
        assert!(resp.contains("deleted"));
        assert!(!dir.join("delete-me.txt").exists());
    }

    #[tokio::test]
    async fn delete_file_not_found() {
        let (port, _dir) = start_test_server().await;
        let resp = tcp_request(port, "DELETE /api/files/nope.txt HTTP/1.1\r\nHost: localhost\r\n\r\n").await;
        assert!(resp.contains("404"), "resp: {}", resp);
    }

    #[tokio::test]
    async fn delete_path_traversal_blocked() {
        let (port, _dir) = start_test_server().await;
        let resp = tcp_request(port, "DELETE /api/files/../../etc/passwd HTTP/1.1\r\nHost: localhost\r\n\r\n").await;
        // Path traversal stripped, becomes etc/passwd which doesn't exist
        assert!(resp.contains("404"), "resp: {}", resp);
    }

    #[tokio::test]
    async fn serve_static_file() {
        let (port, dir) = start_test_server().await;
        std::fs::write(dir.join("index.html"), "<h1>Hello</h1>").unwrap();
        let resp = tcp_request(port, "GET /index.html HTTP/1.1\r\nHost: localhost\r\n\r\n").await;
        assert!(resp.contains("200"), "resp: {}", resp);
    }

    #[tokio::test]
    async fn serve_index_html_for_directory() {
        let (port, dir) = start_test_server().await;
        std::fs::write(dir.join("index.html"), "<h1>Root</h1>").unwrap();
        let resp = tcp_request(port, "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n").await;
        assert!(resp.contains("200"), "resp: {}", resp);
    }

    #[tokio::test]
    async fn router_without_site_dir_has_fallback() {
        let app = f8(test_state(false));
        let resp = app.oneshot(req("GET", "/")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = String::from_utf8(
            resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
        ).unwrap();
        assert!(body.contains("No cloud. No hosting bill. Ever."));
    }

    #[tokio::test]
    async fn router_with_site_dir_no_fallback() {
        let app = f8(test_state(true));
        // With site dir but no files, ServeDir returns 404 for /
        let resp = app.oneshot(req("GET", "/nonexistent-file.xyz")).await.unwrap();
        // ServeDir should return 404 for missing files
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
