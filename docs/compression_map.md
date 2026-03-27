# Compression Map — pocket-server

P13 tokenization. Every public symbol has a compressed identifier.

## Types (t)

| Token | Name | Module | Purpose |
|-------|------|--------|---------|
| t0 | AppState | server | Shared server state: stats, site name, hostname, site dir |
| t1 | Stats | stats | Live request/byte/uptime tracker with atomics |

## Functions (f)

| Token | Name | Module | Signature |
|-------|------|--------|-----------|
| f0 | dashboard_page | server | fn(state: &t0) -> String |
| f1 | landing_page | server | fn(state: &t0) -> String |
| f2 | dashboard | server | async handler — GET /dashboard |
| f3 | fallback_index | server | async handler — GET / (no site dir) |
| f4 | api_stats | server | async handler — GET /api/stats |
| f5 | health | server | async handler — GET /health |
| f6 | upload | server | async handler — POST /api/upload (localhost only) |
| f7 | counting_layer | server | async middleware — counts bytes on static file responses |
| f8 | build_router | server | pub fn(state: t0) -> Router |
| f9 | run | server | pub async fn(s1, s2, port, s3) — start server, blocking |
| f10 | Stats::new | stats | pub fn() -> t1 |
| f11 | record_request | stats | pub fn(&self, bytes: u64) |
| f12 | uptime_secs | stats | pub fn(&self) -> u64 |
| f13 | uptime_display | stats | pub fn(&self) -> String — "Xh Ym" |
| f14 | requests_total | stats | pub fn(&self) -> u64 |
| f15 | bytes_total | stats | pub fn(&self) -> u64 |
| f16 | bytes_display | stats | pub fn(&self) -> String — "X KB" |
| f17 | power_estimate_w | stats | pub fn(&self) -> f64 — watts |
| f18 | monthly_cost_display | stats | pub fn(&self) -> String — "$X.XX" |
| f19 | to_json | stats | pub fn(&self) -> String — JSON snapshot |
| f20 | start | tunnel | pub async fn(port: u16) — spawn cloudflared |
| f21 | parse_args | main | fn() -> (String, u16, Option<PathBuf>, bool) |
| f22 | get_runtime | android | fn() -> &'static Runtime — lazy tokio init |

## Fields (s)

| Token | Name | Owner | Type |
|-------|------|-------|------|
| s0 | stats | t0 | Arc<t1> |
| s1 | site_name | t0 | String |
| s2 | hostname | t0 | String |
| s3 | site_dir | t0 | Option<PathBuf> |
| s4 | start | t1 | Instant |
| s5 | requests | t1 | AtomicU64 |
| s6 | bytes_served | t1 | AtomicU64 |

## Routes

| Path | Method | Handler | Auth |
|------|--------|---------|------|
| / | GET | f3 (fallback) or ServeDir | public |
| /dashboard | GET | f2 | public |
| /api/stats | GET | f4 | public |
| /api/upload | POST | f6 | localhost only |
| /health | GET | f5 | public |

## Notes

- JNI symbols in android.rs cannot be renamed (Java naming convention)
- `main()` kept as `main` (Rust entry point convention)
- JSON in f19 is hand-formatted (no serde dependency)
