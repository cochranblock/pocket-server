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

## Stats

- **Binary:** 1.01 MB (release, stripped, LTO)
- **Rust LOC:** 593
- **Direct deps:** 3 (axum, tokio, tower-http)
- **Power estimate:** 0.5W idle + 0.1W/req/sec
- **Monthly cost:** ~$0.05 at idle ($0.15/kWh)

## License

[Unlicense](UNLICENSE) — public domain.

Part of the [CochranBlock](https://cochranblock.org) zero-cloud architecture.
