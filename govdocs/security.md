# Security Posture — Pocket Server

## Architecture

Pocket Server is a single compiled Rust binary. No interpreter, no VM, no dynamic linking of application code. The attack surface is the compiled binary and the OS network stack.

## Memory Safety

Written in Rust. Memory safety enforced at compile time by the borrow checker. No use-after-free, no buffer overflows, no null pointer dereferences in safe code. One `unsafe` block in `ios.rs` (`CStr::from_ptr` for FFI string conversion). JNI and iOS bridges use `#[unsafe(no_mangle)]` for FFI symbol export.

## Dependencies

3 direct dependencies, all from the Rust ecosystem:
- **axum** (0.7) — HTTP framework by the Tokio team
- **tokio** (1.x) — async runtime, industry standard
- **tower-http** (0.5) — HTTP middleware (compression, static files)

Full SBOM available at `/govdocs/sbom` or via `--sbom` flag.

## Network Exposure

| Port | Protocol | Purpose |
|------|----------|---------|
| 8080 (configurable) | HTTP | Site serving, dashboard, stats API |

## Access Controls

- **Upload endpoint** (`POST /api/upload`): restricted to localhost (loopback) via `ConnectInfo<SocketAddr>` check. Remote clients receive 403 Forbidden.
- **Path traversal protection**: upload filenames are sanitized — `..` components stripped, paths normalized. Files cannot escape the site directory.
- **No authentication on read endpoints**: site content, dashboard, and stats are public. This is by design — the server hosts a public website.

## Data at Rest

- Site files stored unencrypted on phone storage (app-private directory on Android)
- No database. No secrets stored by the server.
- Stats are in-memory only (atomic counters). Lost on restart.

## Data in Transit

- HTTP by default (no TLS termination in the binary)
- When `--tunnel` is used, traffic flows through Cloudflare's network with TLS termination at Cloudflare's edge
- zstd compression on all responses

## Supply Chain

- All source code is Unlicense (public domain)
- Repository: https://github.com/cochranblock/pocket-server
- No pre-built binaries distributed — customers build from source or receive a pre-configured device
- Cargo.lock pins all dependency versions

## Audit History

Last audited via [P23 Triple Lens](https://github.com/cochranblock/kova/blob/main/docs/KOVA_BLUEPRINT.md#10-p23-triple-lens-research-protocol) on 2026-04-02. Paranoia lens confirmed: upload auth PASS, path traversal PASS. Red flags: no rate limiting, no request logging, unbounded upload size.

## Known Limitations

- No TLS termination in the binary itself (relies on tunnel or reverse proxy)
- No rate limiting
- No request logging to disk
- Upload size is unbounded (limited only by phone storage)

---

Product of [CochranBlock](https://cochranblock.org) zero-cloud architecture.
