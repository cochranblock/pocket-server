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
}
