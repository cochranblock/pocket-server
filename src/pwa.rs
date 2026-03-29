// Unlicense — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.6

//! PWA support — manifest.json, service worker, icon endpoints.
//! Makes Pocket Server installable from any browser.

use axum::response::IntoResponse;

/// f27=manifest_json — GET /manifest.json
pub async fn f27() -> impl IntoResponse {
    (
        [("content-type", "application/manifest+json")],
        concat!(
            r#"{"name":"Pocket Server","short_name":"PocketSrv","#,
            r#""description":"Your website lives on your phone. No hosting bill. Ever.","#,
            r#""start_url":"/dashboard","display":"standalone","orientation":"portrait","#,
            r##""theme_color":"#0a0a0a","background_color":"#0a0a0a","##,
            r#""icons":[{"src":"/pwa/icon.svg","sizes":"any","type":"image/svg+xml"}]}"#
        ),
    )
}

/// f28=service_worker — GET /sw.js
pub async fn f28() -> impl IntoResponse {
    (
        [("content-type", "application/javascript")],
        concat!(
            "const CACHE='pocket-server-v1';\n",
            "const ASSETS=['/','/dashboard','/api/stats','/manifest.json'];\n",
            "self.addEventListener('install',e=>{e.waitUntil(caches.open(CACHE).then(c=>c.addAll(ASSETS)));self.skipWaiting()});\n",
            "self.addEventListener('activate',e=>{e.waitUntil(caches.keys().then(ks=>Promise.all(ks.filter(k=>k!==CACHE).map(k=>caches.delete(k)))));self.clients.claim()});\n",
            "self.addEventListener('fetch',e=>{if(e.request.url.includes('/api/')){e.respondWith(fetch(e.request).catch(()=>caches.match(e.request)))}else{e.respondWith(caches.match(e.request).then(r=>r||fetch(e.request).then(resp=>{caches.open(CACHE).then(c=>c.put(e.request,resp.clone()));return resp})))}});\n",
        ),
    )
}

/// f29=pwa_icon — GET /pwa/icon.svg
pub async fn f29() -> impl IntoResponse {
    (
        [("content-type", "image/svg+xml")],
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512">"#,
            r##"<rect width="512" height="512" rx="96" fill="#0a0a0a"/>"##,
            r##"<rect x="156" y="80" width="200" height="352" rx="24" fill="#00d4aa"/>"##,
            r##"<rect x="172" y="112" width="168" height="272" fill="#0a0a0a"/>"##,
            r##"<circle cx="256" cy="208" r="20" fill="#00d4aa"/>"##,
            r##"<circle cx="256" cy="208" r="48" fill="none" stroke="#00d4aa" stroke-width="10"/>"##,
            r##"<circle cx="256" cy="208" r="80" fill="none" stroke="#00d4aa" stroke-width="10"/>"##,
            r##"<polygon points="244,288 256,264 268,288" fill="#00d4aa"/>"##,
            r##"<rect x="252" y="288" width="8" height="56" fill="#00d4aa"/>"##,
            "</svg>",
        ),
    )
}

/// PWA meta tags to inject into HTML pages.
pub const PWA_HEAD: &str = concat!(
    r#"<link rel="manifest" href="/manifest.json">"#,
    r##"<meta name="theme-color" content="#0a0a0a">"##,
    r#"<link rel="apple-touch-icon" href="/pwa/icon.svg">"#,
    r#"<meta name="apple-mobile-web-app-capable" content="yes">"#,
    r#"<meta name="apple-mobile-web-app-status-bar-style" content="black">"#,
);
