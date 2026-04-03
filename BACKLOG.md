<!-- Unlicense — cochranblock.org -->

# Backlog

Prioritized work items for pocket-server. Most important at top. Max 20.

> Tags: `[build]` `[test]` `[fix]` `[feature]` `[docs]` `[research]`
>
> Cross-project deps in **bold**. This backlog self-reorganizes based on recency and relevance.

---

1. ~~`[test]` Add Rust tests~~ **DONE** — 27 tests: 8 stats unit, 14 route integration (all 12 endpoints + upload auth + localhost TCP), 5 govdocs unit (SPDX, parser, md_to_html)
2. ~~`[fix]` Graceful shutdown~~ **DONE** — SIGINT/SIGTERM via tokio::signal, with_graceful_shutdown(), stopServer() JNI + Java wired
3. ~~`[feature]` Request logging middleware~~ **DONE** — f30 access log (method path status ms), --quiet/-q flag, quiet by default on Android/iOS
4. ~~`[feature]` File management API~~ **DONE** — f31 GET /api/files (recursive JSON list), f32 DELETE /api/files/*path (localhost only, path sanitized). 29 tests.
5. ~~`[fix]` Upload size limit~~ **DONE** — --max-upload flag (default 50 MB), 413 Payload Too Large on exceed, Android/iOS default 50 MB
6. ~~`[build]` IRONHIVE CI script~~ **DONE** — `ci/build.sh`: test → clippy -D warnings → release build. Deploy to lf (n0). **Depends on [ironhive](https://github.com/cochranblock/ironhive) sync daemon**
7. `[fix]` CSRF protection on upload + delete — browser JS on localhost can POST multipart or DELETE files with no preflight (simple request exemption). Add a startup-generated 32-byte hex secret stored in AppState, required as `X-Pocket-Token` header on all mutating endpoints. Return 403 without it. Expose at `GET /api/token` (loopback-only). Display in CLI banner. Closes the live CSRF hole for desktop users.
8. `[fix]` Rate limiting — per-IP token bucket (60 req/s, burst 120) as axum middleware before routing. Store `DashMap<IpAddr, (u64, Instant)>` in AppState. Return 429. No new deps — implement inline. Prevents LAN attacker from overheating the phone or skewing power stats.
9. `[fix]` Symlink escape in upload + delete — `base.join(clean)` after stripping `..` is not safe if `base` or `clean` contains symlinks pointing outside the site dir. After constructing `dest`, canonicalize the parent and assert it has `base.canonicalize()` as prefix. Return 403 on failure. Apply to both f6 and f32.
10. `[test]` Android integration test — adb install AAB on emulator, verify server starts, dashboard loads, stats API responds. Manual today, automate with `adb shell` + curl
11. `[build]` Deploy Linux binary to gd — pocket-server as a service on gd (n1) behind approuter. Register with approuter's service registry. **Depends on [approuter](https://github.com/cochranblock/approuter) service registration**
12. `[feature]` Config file — `~/.pocket-server.toml` for persistent settings (port, site-dir, name, tunnel, quiet). Flags override config. Removes need to retype args
13. `[feature]` Custom domain via approuter — register pocket-server hostname with approuter for `*.cochranblock.org` subdomain routing. **Depends on [approuter](https://github.com/cochranblock/approuter) hostname API**
14. `[build]` Wire oakilydokily Android to pocket-server — oakilydokily's Android module is a scaffold waiting on this project's JNI bridge stabilization. **Depends on [oakilydokily](https://github.com/cochranblock/oakilydokily) Android WebView**
15. `[feature]` Multi-site support — serve multiple site directories on different paths or virtual hosts. Enables one phone to host several sites
16. `[docs]` API documentation — OpenAPI/Swagger spec for all 12 endpoints. Serve at `/api/docs` or embed in govdocs
17. `[research]` P23 re-audit after tests land — re-run paranoia lens once items 1-7 are complete. Verify red flags resolved, check for new attack surface from added features
18. `[build]` Play Store submission — sign AAB, create store listing, submit for review. Requires privacy policy page. **Depends on [cochranblock](https://github.com/cochranblock/cochranblock) product page**
19. `[research]` kova c2 offload integration — archive build artifacts to IRONHIVE worker storage (bt node, `/mnt/hive/archive`). Keeps Mac Mini clean. **Depends on [kova](https://github.com/cochranblock/kova) `c2 offload` command**
20. `[feature]` Stats persistence — write stats to disk periodically (sled or flat file), survive restarts. Current: in-memory atomic counters, lost on restart
