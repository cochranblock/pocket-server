# Timeline of Invention — pocket-server

Chronological record of every commit. Entries are immutable once pushed.

---

## 2026-03-26 — Project Genesis

| Hash | Description |
|------|-------------|
| `afde6a9` | pocket-server: phone-as-web-server scaffold — Rust server, JNI bridge, Android dashboard |
| `7d6cda1` | add Unlicense |

Initial scaffold: Rust cdylib with Axum, JNI bridge to Android, Stats tracker with atomic counters, power/cost estimation. Three routes: /, /api/stats, /health. PocketServer.java JNI stub.

## 2026-03-27 — Build Day

### Health Check + Bug Fix
| Hash | Description |
|------|-------------|
| `20915ad` | add README with cochranblock.org backlink |
| `2be6b09` | fix: share Stats between JNI global and AppState via Arc |

Found bug: `startServer` created a second `Stats::new()` for AppState instead of sharing the Arc stored in STATS global. `getStats()` from Java always returned zeros. Fixed by making `AppState.stats` an `Arc<Stats>`.

### Feature Build — Dashboard, File Serving, Android Shell
| Hash | Description |
|------|-------------|
| `0e56fb4` | add kiosk dashboard, file serving, Android app shell, Gradle build |

- Live-updating kiosk dashboard at `/dashboard` — polls `/api/stats` every 2s
- User site serving via tower-http `ServeDir` from phone storage
- Android: DashboardActivity (fullscreen WebView), ServerService (foreground service + wake lock), BootReceiver (auto-start on boot)
- AndroidManifest.xml with all required permissions
- Gradle build with cargo-ndk cross-compile task for arm64-v8a + armeabi-v7a
- Removed unused rust-embed dependency

### Feature Build — CLI Binary, Upload, Tunnel
| Hash | Description |
|------|-------------|
| `112d588` | add CLI binary, file upload, Cloudflare tunnel — full demo works |

- `src/main.rs`: CLI with --name, --port, --site-dir, --tunnel flags
- `POST /api/upload`: multipart file upload to site directory
- `src/tunnel.rs`: spawns `cloudflared tunnel --url` as child process
- Cargo.toml: both cdylib (Android) and rlib+bin (desktop), jni is Android-only
- Full demo verified: health, dashboard, stats, upload, serve, fallback

### QA Round 1
| Hash | Description |
|------|-------------|
| `b162d60` | fix: QA audit — clippy, path traversal, upload auth, box art |

- Added `Default` impl for `Stats` (clippy warning)
- Fixed path traversal: strip to filename only via `rsplit`
- Restricted `/api/upload` to localhost via `ConnectInfo<SocketAddr>`
- Fixed tunnel box art to dynamically size to URL length
- `cargo clippy --release -- -D warnings` passes clean

### QA Round 2
- `cargo clean && cargo build --release` — clean, zero errors
- `cargo clippy --release -- -D warnings` — zero warnings
- `git status` — clean, up to date with origin/main
- Result: **PASS**

### P13 Tokenization + Binary Optimization
| Hash | Description |
|------|-------------|
| `6e58cdb` | P13 tokenize all symbols + strip serde deps — 1.01 MB binary |

- All public symbols renamed: t0-t1 (types), f0-f22 (functions), s0-s6 (fields)
- `docs/compression_map.md`: canonical token reference
- Removed unused serde + serde_json from direct dependencies
- Release binary: 1,055,328 bytes (1.01 MB) on aarch64-apple-darwin
- cdylib: 16,752 bytes

### User Story Analysis + Top 3 Fixes
| Hash | Description |
|------|-------------|
| *(pending)* | user story analysis, top 3 fixes, timeline, proof of artifacts |

- Full 10-point user story walkthrough as simulated customer
- Verdict: 2.8/10 — working prototype, not shippable product
- Fix 1: Port-in-use panic replaced with clean error + exit
- Fix 2: Subdirectory upload support (preserves relative paths, blocks traversal)
- Fix 3: README rewritten with quickstart, API docs, stats
- Startup now prints "listening on" message and site URL
- Non-existent site dir auto-created with warning
