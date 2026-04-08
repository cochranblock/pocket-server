# Timeline of Invention — pocket-server

Chronological record of every commit. Entries are immutable once pushed.

---

## Human Revelations — Invented Techniques

*Novel ideas that came from human insight, not AI suggestion. These are original contributions to the field.*

### Phone-as-Web-Server (March 2026)

**Invention:** A Rust web server compiled to a JNI shared library, loaded by an Android foreground service, that turns a phone into a production web server — serving user files, a live stats dashboard, and accepting file uploads, with optional Cloudflare tunnel for internet exposure.

**The Problem:** Personal web hosting requires a computer that's always on, an ISP that allows incoming connections, a domain, and DNS configuration. Raspberry Pi servers exist but require setup, power, and network configuration. Everyone has a phone in their pocket with gigabit connectivity, but phones aren't web servers.

**The Insight:** A phone has everything a web server needs: CPU, storage, network, and power (battery). Android allows foreground services with wake locks. Rust compiles to a shared library that Android loads via JNI. The phone is already always-on and always-connected. It just needs a web server binary — 1.04MB of Rust.

**The Technique:**
1. Rust cdylib compiled via cargo-ndk for arm64-v8a (Android) and armeabi-v7a
2. JNI bridge: `startServer(port, siteDir)` — Java starts the Rust Axum server
3. Foreground service with wake lock: server stays running when screen is off
4. BootReceiver: auto-start server on device boot
5. File serving: tower-http ServeDir from phone storage directory
6. File upload: POST /api/upload with path traversal protection
7. Stats dashboard: live-updating kiosk page polling /api/stats every 2s
8. Cloudflare tunnel: optional `cloudflared tunnel --url` for internet exposure
9. CLI binary for desktop: same Rust code, different entry point

**Result:** A 737KB AAB turns any Android phone into a web server. Upload files from a laptop, serve them to the internet via Cloudflare tunnel. The phone in your pocket is the server.

**Named:** Pocket Server
**Commit:** `afde6a9` (initial scaffold), `112d588` (CLI + tunnel)
**Origin:** The question: "What's the smallest possible web host?" Not a Raspberry Pi. Not a VPS. The phone that's already in your pocket, already powered on, already connected to the internet.

### 2026-04-08 — Human Revelations Documentation Pass

**What:** Documented novel human-invented techniques across the full CochranBlock portfolio. Added Human Revelations section with Phone-as-Web-Server.
**Commit:** See git log
**AI Role:** AI formatted and wrote the sections. Human identified which techniques were genuinely novel, provided the origin stories, and directed the documentation pass.

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
| `4f4479b` | user story analysis, top 3 fixes, timeline, proof of artifacts |

- Full 10-point user story walkthrough as simulated customer
- Verdict: 2.8/10 — working prototype, not shippable product
- Fix 1: Port-in-use panic replaced with clean error + exit
- Fix 2: Subdirectory upload support (preserves relative paths, blocks traversal)
- Fix 3: README rewritten with quickstart, API docs, stats
- Startup now prints "listening on" message and site URL
- Non-existent site dir auto-created with warning

| `718e982` | update timeline with commit hash for 4f4479b |

## 2026-03-28 — Compliance + Packaging

### Govdocs + SBOM
| Hash | Description |
|------|-------------|
| `4bbcea0` | add /govdocs routes + --sbom flag — binary is its own compliance artifact |

- GET /govdocs, /govdocs/sbom, /govdocs/capability, /govdocs/security
- SBOM live from baked Cargo.lock (98 transitive deps)
- --sbom flag dumps SPDX 2.3 to stdout
- CRT theme (dark, monospace, #00d4aa)
- govdocs/capability.md + govdocs/security.md baked via include_str!

### Crates.io Prep
| Hash | Description |
|------|-------------|
| `710bac1` | add keywords + categories to Cargo.toml for crates.io |

- `cargo publish --dry-run` passes clean

## 2026-03-29 — Android API 35 + Multi-Platform + AAB

### Android API 35
| Hash | Description |
|------|-------------|
| `25ea8b2` | android: target API 35, fix all deprecations, add app icon |

- compileSdk/targetSdk 34 → 35, AGP 8.7.3, Gradle 8.10.2
- WindowInsetsController for immersive mode (replaces deprecated API)
- Runtime POST_NOTIFICATIONS permission (API 33+)
- Adaptive vector app icon, proguard-rules.pro, wake lock timeout fix

### iOS + PWA + Cross-Compilation
| Hash | Description |
|------|-------------|
| `a3b965c` | add iOS app, PWA support, cross-compilation for 13 targets |

- iOS: Swift AppDelegate with @_silgen_name FFI, WKWebView, Info.plist
- PWA: manifest.json, sw.js, SVG icon, service worker registration
- build-all-targets.sh for 13 architectures
- Cargo.toml: staticlib added to crate-type

### Real AAB Built
| Hash | Description |
|------|-------------|
| `8ad007d` | build real AAB: jni 0.22 API migration + gradle bundleRelease |

- Migrated android.rs from deprecated JNIEnv to EnvUnowned + with_env()
- cargo ndk -t arm64-v8a: 1,722,544 byte .so (ELF ARM64)
- gradle bundleRelease: BUILD SUCCESSFUL, 43 tasks
- AAB: 737,815 bytes — ready for Play Store upload

### Truth Audit
| Hash | Description |
|------|-------------|
| `cee0422` | truth audit: fix 15 discrepancies across docs |

- 15 discrepancies found and fixed across README, PROOF_OF_ARTIFACTS,
  compression_map.md, govdocs/security.md
- 2 lies (ARM32 claimed but dropped, "zero unsafe" but ios.rs has one)
- 5 timeline commits spot-checked — all verified

## 2026-03-30 — Polish Pass

| Hash | Description |
|------|-------------|
| `d8c9220` | polish: TOI current, empty assets/ removed, .gitignore hardened |

- TOI brought current with all 7 missing commits
- Removed empty `assets/` directory (dead weight from scaffold)
- .gitignore hardened: added .DS_Store, *.log, *.env
- cargo audit: 0 advisories / 99 deps
- cargo outdated: axum 0.7→0.8, tower-http 0.5→0.6 available (major bumps, left as-is)
- cargo tree --duplicates: 0 duplicates
- Working tree clean, clippy clean

## 2026-03-31 — TOI + POA Update

| Hash | Description |
|------|-------------|
| `0b1eec9` | update TOI + POA: add d8c9220 polish commit, verify date 2026-03-31 |

- Timeline and Proof of Artifacts brought current with polish pass commit

## 2026-04-02 — P23 Triple Lens Audit + IRONHIVE Swarm Build

| Hash | Description |
|------|-------------|
| `f4943e4` | README: add iOS section, fix platform types, add Linux binary size + LOC stats |
| `5e66b18` | docs: accuracy pass + cochranblock.org cross-links across all docs |

**Method:** P23 Triple Lens — full guest analysis applied three opposing perspectives to pocket-server.

| Lens | Focus | Key Findings |
|------|-------|-------------|
| **Optimist** | What works, competitive advantages | 1,063 LOC Rust, 3 deps, 1.04 MB binary, 12 routes, real AAB, 13 build targets. Solid engineering for a prototype. |
| **Pessimist** | Gaps, missing features, UX failures | Zero tests, no logging, no graceful shutdown, no file list/delete API, no TLS, iOS has no Xcode project. Product score: 2.8/10. |
| **Paranoia** | Security risks, attack vectors, failure modes | Path traversal protection verified (PASS). Upload auth verified (PASS). No rate limiting (risk). Unbounded uploads (risk). No request logging (blind spot). |
| **Synthesis** | Priority-ordered action items | Tests → graceful shutdown → logging → IRONHIVE CI → file management API → TLS → iOS Xcode |

- Synced pocket-server to IRONHIVE cluster (lf, gd, st) via rsync
- Built x86_64-unknown-linux-gnu release on node lf (n0): 1,485,656 bytes (1.42 MB), 18s
- Built aarch64-apple-darwin release locally: 1,088,560 bytes (1.04 MB), 7s
- cochranblock.org cross-linked across all 7 doc files
- P23 reference: [KOVA Blueprint](https://github.com/cochranblock/kova/blob/main/docs/KOVA_BLUEPRINT.md#10-p23-triple-lens-research-protocol)

---

Part of [CochranBlock](https://cochranblock.org) zero-cloud architecture.
