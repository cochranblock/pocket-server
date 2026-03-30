# Proof of Artifacts — pocket-server

Measurable state of the project. Updated each session.

Last verified: 2026-03-29

---

## Binary Size

| Target | Size | Date |
|--------|------|------|
| aarch64-apple-darwin (release bin) | 1,088,560 bytes (1.04 MB) | 2026-03-29 |
| x86_64-apple-darwin (release bin) | 1,221,284 bytes (1.16 MB) | 2026-03-29 |
| aarch64-linux-android (.so) | 1,722,544 bytes (1.64 MB) | 2026-03-29 |
| Android AAB (app-release.aab) | 737,815 bytes (721 KB) | 2026-03-29 |

Profile: `opt-level=z`, `lto=true`, `codegen-units=1`, `strip=true`, `panic=abort`

## Lines of Code

| File | LOC | Purpose |
|------|-----|---------|
| govdocs.rs | 319 | Compliance docs, SBOM, SPDX generator |
| server.rs | 287 | Router, handlers, upload, middleware |
| stats.rs | 108 | Atomic counters, power/cost estimation |
| android.rs | 96 | JNI bridge (cfg android only) |
| main.rs | 94 | CLI entry point, arg parsing |
| pwa.rs | 63 | PWA manifest, service worker, icon |
| tunnel.rs | 48 | Cloudflare tunnel child process |
| ios.rs | 30 | iOS FFI entry point (cfg ios only) |
| lib.rs | 18 | Module declarations |
| **Rust total** | **1,063** | |
| Java (4 files) | 250 | Activity, Service, Receiver, JNI |
| **Grand total** | **1,313** | |

## Function Count

| Module | Functions | Tokens |
|--------|-----------|--------|
| server.rs | 10 | f0-f9 |
| stats.rs | 11 | f10-f19 (+ Default impl) |
| tunnel.rs | 1 | f20 |
| main.rs | 2 | f21 + main |
| android.rs | 3 | f22 + 2 JNI exports |
| govdocs.rs | 9 | f23-f26 + 5 internal helpers |
| pwa.rs | 3 | f27-f29 |
| ios.rs | 1 | pocket_server_ios_main |
| **Total** | **40** | |

## Type Count

| Token | Name | Module |
|-------|------|--------|
| t0 | AppState | server.rs |
| t1 | Stats | stats.rs |

## Field Count

| Token | Name | Owner |
|-------|------|-------|
| s0 | stats | t0 |
| s1 | site_name | t0 |
| s2 | hostname | t0 |
| s3 | site_dir | t0 |
| s4 | start | t1 |
| s5 | requests | t1 |
| s6 | bytes_served | t1 |

## Dependencies

| Type | Count |
|------|-------|
| Direct (Cargo.toml) | 3 (axum, tokio, tower-http) |
| Android-only | 1 (jni) |
| Transitive (Cargo.lock) | 98 |

## P13 Tokenization Stats

- Types tokenized: 2 (t0-t1)
- Functions tokenized: 30 (f0-f29)
- Fields tokenized: 7 (s0-s6)
- Error variants: 0 (no custom error enum)
- Compression map: `docs/compression_map.md`

## QA Results

### QA Round 1 — 2026-03-27

| Check | Result |
|-------|--------|
| `cargo build --release` | PASS — zero errors |
| `cargo clippy --release -- -D warnings` | PASS — zero warnings (after fixing `new_without_default`) |
| Path traversal test | PASS — file stays in site dir |
| Upload auth test | PASS — localhost only, remote rejected with 403 |
| Full route regression | PASS — health, dashboard, stats, upload, serve, fallback |
| **Verdict** | **PASS** |

### QA Round 2 — 2026-03-27

| Check | Result |
|-------|--------|
| `cargo clean && cargo build --release` | PASS — clean build, zero errors, zero warnings |
| `cargo clippy --release -- -D warnings` | PASS — zero warnings |
| Test binary (`--features tests`) | N/A — no test feature in this crate |
| `git status` | Clean — nothing to commit |
| `git log -1 --oneline` | `b162d60` — correct, matches QA round 1 fixes |
| **Verdict** | **PASS** |

## User Story Analysis — 2026-03-27

| Category | Score (1-10) |
|----------|-------------|
| Usability | 5 |
| Completeness | 3 |
| Error Handling | 3 |
| Documentation | 1 |
| Would You Pay $500? | 2 |
| **Average** | **2.8** |

Full analysis: `USER_STORY_ANALYSIS.md`

### Top 3 Fixes Applied

1. Port-in-use panic → clean error message + exit 1
2. Subdirectory upload → preserves relative paths, creates dirs, blocks traversal
3. README → full quickstart, API docs, CLI flags, binary stats

## Routes (12 total)

| Path | Method | Handler | Auth |
|------|--------|---------|------|
| / | GET | f3 (fallback) or ServeDir | public |
| /dashboard | GET | f2 | public |
| /api/stats | GET | f4 | public |
| /api/upload | POST | f6 | localhost only |
| /health | GET | f5 | public |
| /govdocs | GET | f23 | public |
| /govdocs/sbom | GET | f24 | public |
| /govdocs/capability | GET | f25 | public |
| /govdocs/security | GET | f26 | public |
| /manifest.json | GET | f27 | public |
| /sw.js | GET | f28 | public |
| /pwa/icon.svg | GET | f29 | public |
