# Pocket Server

Your website lives on your phone. No hosting bill. Ever.

A compiled Rust web server that runs on Android (or any machine). Serves static sites from local storage with a live kiosk dashboard showing uptime, requests, bytes, power draw, and estimated monthly cost.

## Quickstart

```
git clone https://github.com/cochranblock/pocket-server
cd pocket-server
cargo build --release
```

### Serve a site

```
mkdir my-site
echo '<h1>Hello world</h1>' > my-site/index.html
./target/release/pocket-server --site-dir my-site
```

Output:
```
pocket-server v0.1.0
  name:     Pocket Server
  port:     8080
  site-dir: my-site
  site:      http://127.0.0.1:8080/
  dashboard: http://127.0.0.1:8080/dashboard
  listening on 0.0.0.0:8080
```

### Upload files via API

```
curl -F "file=@index.html;filename=index.html" http://localhost:8080/api/upload
curl -F "file=@style.css;filename=css/style.css" http://localhost:8080/api/upload
```

Uploads are localhost-only. Subdirectories are created automatically.

### Expose to the internet

```
./target/release/pocket-server --site-dir my-site --tunnel
```

Requires [cloudflared](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/). Gives you a public `*.trycloudflare.com` URL.

## CLI Flags

```
--name, -n <name>      Site name (default: Pocket Server)
--port, -p <port>      Port to bind (default: 8080)
--site-dir, -d <path>  Directory with site files to serve
--tunnel, -t           Start Cloudflare quick tunnel
--help, -h             Help
```

Without `--site-dir`, a default landing page is served.

## API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Your site (or default landing page) |
| `/dashboard` | GET | Live stats dashboard (kiosk UI) |
| `/api/stats` | GET | JSON: uptime, requests, bytes, watts, cost |
| `/api/upload` | POST | Upload files (multipart, localhost only) |
| `/health` | GET | Returns "OK" |

## Android

The server compiles as a shared library (`libpocket_server.so`) for Android. The `android/` directory contains:

- **DashboardActivity** — fullscreen WebView kiosk dashboard
- **ServerService** — foreground service with wake lock
- **BootReceiver** — auto-start on reboot

Build: `cd android && ./gradlew assembleRelease` (requires cargo-ndk + Android NDK).

## PWA

The dashboard is installable as a Progressive Web App from any browser. Open `/dashboard`, tap "Add to Home Screen." Works offline after first load.

## Platforms

| Platform | Target | Type | Status |
|----------|--------|------|--------|
| macOS ARM | `aarch64-apple-darwin` | Binary | Ready |
| macOS Intel | `x86_64-apple-darwin` | Binary | Ready |
| Linux x86_64 | `x86_64-unknown-linux-gnu` | Binary | Cross-compile via `cross` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | Binary | Cross-compile (RPi 4/5, Graviton) |
| Linux ARM32 | `armv7-unknown-linux-gnueabihf` | Binary | Cross-compile (older RPi, IoT) |
| Android ARM64 | `aarch64-linux-android` | APK | cargo-ndk + Gradle |
| Android ARM32 | `armv7-linux-androideabi` | APK | cargo-ndk + Gradle |
| iOS | `aarch64-apple-ios` | IPA | Xcode + staticlib |
| Windows | `x86_64-pc-windows-gnu` | Binary | Cross-compile via `cross` |
| FreeBSD | `x86_64-unknown-freebsd` | Binary | Cross-compile via `cross` |
| RISC-V | `riscv64gc-unknown-linux-gnu` | Binary | Cross-compile via `cross` |
| POWER | `powerpc64le-unknown-linux-gnu` | Binary | Cross-compile (gov mainframes) |
| Web (PWA) | Browser | Installable | Built-in, any device |

Build all: `./build-all-targets.sh`

## Stats

- **Binary:** ~1.1 MB (release, stripped, LTO)
- **Direct deps:** 3 (axum, tokio, tower-http)
- **Power estimate:** 0.5W idle + 0.1W/req/sec
- **Monthly cost:** ~$0.05 at idle ($0.15/kWh)

## Compliance

The binary serves its own compliance docs at runtime:
- `/govdocs` — index
- `/govdocs/sbom` — live SBOM from Cargo.lock
- `/govdocs/capability` — capability statement
- `/govdocs/security` — security posture
- `--sbom` — SPDX 2.3 to stdout

## License

[Unlicense](UNLICENSE) — public domain.

Part of the [CochranBlock](https://cochranblock.org) zero-cloud architecture.
