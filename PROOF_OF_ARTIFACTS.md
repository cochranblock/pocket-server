# Proof of Artifacts — pocket-server

Measurable state of the project. Updated each session.

---

## Binary Size

| Target | Size | Date |
|--------|------|------|
| aarch64-apple-darwin (release) | 1,055,328 bytes (1.01 MB) | 2026-03-27 |
| cdylib (.dylib, macOS) | 16,752 bytes (16 KB) | 2026-03-27 |

Profile: `opt-level=z`, `lto=true`, `codegen-units=1`, `strip=true`, `panic=abort`

## Lines of Code

| File | LOC | Purpose |
|------|-----|---------|
| server.rs | 273 | Router, handlers, upload, middleware |
| stats.rs | 108 | Atomic counters, power/cost estimation |
| main.rs | 89 | CLI entry point, arg parsing |
| android.rs | 84 | JNI bridge (cfg android only) |
| tunnel.rs | 48 | Cloudflare tunnel child process |
| lib.rs | 13 | Module declarations |
| **Rust total** | **615** | |
| Java (4 files) | 221 | Activity, Service, Receiver, JNI |
| **Grand total** | **836** | |

## Function Count

| Module | Functions | Tokens |
|--------|-----------|--------|
| server.rs | 10 | f0-f9 |
| stats.rs | 11 | f10-f19 (+ Default impl) |
| tunnel.rs | 1 | f20 |
| main.rs | 2 | f21 + main |
| android.rs | 3 | f22 + 2 JNI exports |
| **Total** | **27** | |

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
| Transitive (Cargo.lock) | 99 |

## P13 Tokenization Stats

- Types tokenized: 2 (t0-t1)
- Functions tokenized: 23 (f0-f22)
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

## Routes

| Path | Method | Handler | Auth |
|------|--------|---------|------|
| / | GET | f3 (fallback) or ServeDir | public |
| /dashboard | GET | f2 | public |
| /api/stats | GET | f4 | public |
| /api/upload | POST | f6 | localhost only |
| /health | GET | f5 | public |
