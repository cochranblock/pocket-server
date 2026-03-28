# User Story Analysis — Pocket Server

Simulated by a first-time user. 2026-03-27.

---

## 1. Discovery

**How I found it:** Listed on cochranblock.org/products under "Hardware" as "Coming Soon" at $500-$1000.

**README first impression:** Four lines. "Your website lives on your phone. Rust web server + Android kiosk dashboard." Then stack, license, and a link to cochranblock.org.

**10-second clarity:** Partial. I know it's a phone web server, but:
- How do I install it?
- How do I run it?
- What does the output look like?
- Is this an Android app or a CLI tool?

The README tells me what it IS but not what to DO. No quickstart, no screenshot, no `cargo run` example. A $500 product page links here — this README would lose a buyer in 5 seconds.

**Verdict:** F. The README is a tweet, not documentation for a product.

---

## 2. Installation

```
git clone https://github.com/cochranblock/pocket-server
cd pocket-server
cargo build --release    # works, 17s
./target/release/pocket-server --help
```

**Result:** Works. Help text is clear and minimal:
```
pocket-server — your website lives on your phone

  --name, -n <name>      Site name (default: Pocket Server)
  --port, -p <port>      Port to bind (default: 8080)
  --site-dir, -d <path>  Directory with site files to serve
  --tunnel, -t           Start Cloudflare quick tunnel
  --help, -h             This message
```

**Friction:**
- No install instructions in README. I had to guess `cargo build --release`.
- Binary name is `pocket-server` but Cargo.toml lib is `pocket_server`. Confusing if you're looking for the binary.
- No mention that you need `cloudflared` installed for `--tunnel` until you try it.

**Verdict:** Build works clean. Help text is good. But a user shouldn't need to read Cargo.toml to figure out the install steps.

---

## 3. First Use — Happy Path

**Goal:** Serve my static site from this tool.

**Steps:**
1. Create site: `mkdir ~/my-site && echo '<h1>Hello</h1>' > ~/my-site/index.html`
2. Run: `./pocket-server --site-dir ~/my-site --port 8080`
3. Visit `http://localhost:8080/` in browser
4. Visit `http://localhost:8080/dashboard` for stats

**Result:** All four steps worked perfectly. Site served immediately, dashboard showed live-updating stats with uptime, request count, bytes served, watts, and monthly cost.

**Friction:**
- The startup message says `dashboard: http://127.0.0.1:8080/dashboard` which is helpful.
- But it doesn't say "your site is live at http://127.0.0.1:8080/" — I had to guess.
- No signal when the server is ready (it just goes silent). A "listening on 0.0.0.0:8080" line would help.
- Without `--site-dir`, it serves a landing page that says "This site is served from a phone." That's the Pocket Server default page, not my site. The distinction isn't obvious to a new user who forgot the flag.

**Verdict:** Happy path works. Minimal friction once you know the flags.

---

## 4. Second Use Case — Upload Files Remotely

**Goal:** Push site files to the server without SSH/USB.

**Steps:**
1. Server is running with `--site-dir /tmp/site`
2. `curl -F "file=@index.html;filename=index.html" http://localhost:8080/api/upload`
3. Response: `1 file(s) uploaded`
4. Visit site — new content is live immediately.

**Result:** Works. Multi-file upload works. Files land in the site directory.

**Friction:**
- No documentation of the upload API anywhere. I had to read the source.
- No way to upload to subdirectories. Filenames with paths are stripped to just the basename. So you can't upload `css/style.css` — it becomes `style.css` in the root. This is a dealbreaker for any real site with directory structure.
- No way to list or delete uploaded files via the API.
- No upload size limit. A user could fill the phone storage.

**Verdict:** Upload works for flat sites. Breaks for any site with subdirectories, which is every real site.

---

## 5. Edge Cases

| # | Test | Result | Grade |
|---|------|--------|-------|
| 1 | Invalid port (`--port abc`) | Silently falls back to 8080. No error message. | D |
| 2 | Port already in use | **PANIC** — thread panicked with raw Rust error. No human-readable message. | F |
| 3 | Unknown flag (`--foobar`) | "unknown arg: --foobar", exit 1. | A |
| 4 | Non-existent site dir | Starts fine, 404s on all requests. No warning. | D |
| 5 | Empty POST to /api/upload | "Invalid boundary for multipart/form-data request" — axum internal error. | C |
| 6 | Path traversal (`../../etc/shadow`) | Stripped to "shadow", landed in site dir. Traversal blocked. | B |
| 7 | GET on /api/upload | 405 Method Not Allowed. Correct. | A |
| 8 | Non-existent path | 404. Correct. | A |
| 9 | Port overflow (`--port 99999`) | Silently wraps to 8080. No error. | D |
| 10 | Site name with spaces | Works fine. | A |

**Critical:** Port-in-use panic is unacceptable for a $500 product. A customer would see a Rust stack trace.

---

## 6. Feature Gap Analysis

What a user paying $500-$1000 would expect that doesn't exist:

1. **Subdirectory upload** — Can't upload `css/style.css` or `images/photo.jpg`. Every real website has subdirs.
2. **HTTPS/TLS** — Server is HTTP only. Even localhost should have an option for TLS.
3. **Custom domain** — The $1000 tier promises "domain + ready to plug in." No domain configuration exists.
4. **Site management UI** — No way to browse/delete files except the filesystem. A web UI for managing the site would be expected.
5. **Bandwidth/storage limits** — No protection against filling the phone storage.
6. **Access logs** — No request logging. When something goes wrong, no way to debug.
7. **Multiple sites** — Can only serve one site. No virtual hosts.
8. **Automatic HTTPS via tunnel** — The Cloudflare tunnel gives HTTPS, but only via `*.trycloudflare.com` random URLs, not custom domains.
9. **Restart/update mechanism** — No way to reload config or update the site without restarting.
10. **Status page** — Dashboard is great but only on the phone. No remote status endpoint beyond `/api/stats` JSON.

---

## 7. Documentation Gaps

Questions a user would have that nothing answers:

1. How do I install this on an Android phone?
2. What Android phones are supported?
3. How do I get my domain to point to my phone?
4. What's the bandwidth capacity? How many concurrent users?
5. What happens when the phone loses internet?
6. How do I update my site after initial deployment?
7. What's the upload API format? (No API docs anywhere)
8. How much storage does the server use?
9. Can I run other services alongside it?
10. Where do I get support?

---

## 8. Competitor Check

| Product | Type | Cost | Static Serving | Dashboard | Tunnel |
|---------|------|------|----------------|-----------|--------|
| **Pocket Server** | Rust binary on Android | $500-1000 | Yes | Yes (kiosk) | cloudflared |
| **KSWEB** | Android app (PHP/MySQL/lighttpd) | Free/$5 | Yes | Basic | No |
| **Servers Ultimate** | Android app (multi-server) | Free/$10 | Yes | Yes | No |
| **Termux + nginx** | Manual Linux setup | Free | Yes | No | Manual |
| **GitHub Pages** | Cloud hosted | Free | Yes | No | Built-in |
| **Netlify** | Cloud hosted | Free | Yes | Yes | Built-in |
| **Cloudflare Pages** | Cloud hosted | Free | Yes | Yes | Built-in |

**Honest assessment:**

The free cloud options (GitHub Pages, Netlify, Cloudflare Pages) do everything Pocket Server does for $0 with zero maintenance, automatic HTTPS, global CDN, and CI/CD integration. They don't need a phone, a power cable, or a Cloudflare tunnel.

The value proposition of Pocket Server is ideological, not practical: "your data on your hardware, no cloud dependency." That's a real value for a specific audience (privacy-conscious, self-hosting enthusiasts, off-grid). But the product needs to be EASIER than the free alternative, not harder. Right now it's harder.

KSWEB and Servers Ultimate already exist, are mature, have GUIs, and cost $5-10. They run on the same phones. Pocket Server's advantages: compiled Rust (faster, smaller), kiosk dashboard (nicer), and the hardware appliance model (phone comes pre-configured).

The $500-1000 price makes sense ONLY as a turnkey appliance: phone + configured server + domain + tunnel + plug-and-play. As software alone, it's a tough sell against free alternatives.

---

## 9. Verdict

| Category | Score (1-10) | Notes |
|----------|-------------|-------|
| **Usability** | 5 | Works once you know the flags, but no docs to get you there |
| **Completeness** | 3 | Serves flat static files. No subdirs, no HTTPS, no site management |
| **Error Handling** | 3 | Panics on port-in-use. Silent failures on bad input. |
| **Documentation** | 1 | README is 4 lines. No quickstart, no API docs, no install guide |
| **Would You Pay $500?** | 2 | Not today. The idea is strong. The product isn't ready. |

**Overall: 2.8 / 10**

The server works. The dashboard looks great. The upload API exists. But it panics on common errors, can't handle real websites with subdirectories, has no documentation, and competes against free alternatives. This is a working prototype, not a shippable product.

---

## 10. Top 3 Fixes

These are the minimum changes to move from "prototype" to "demoable":

### Fix 1: Port-in-use panic → clean error message
The server panics with a Rust stack trace when the port is taken. This is the single worst user experience bug. Replace the unwrap with an error message and clean exit.

### Fix 2: Subdirectory upload support
Can't upload `css/style.css` — it strips the path and drops the file in the root. Every real website has subdirectories. The upload must preserve relative paths and create dirs as needed.

### Fix 3: README with quickstart
A $500 product with a 4-line README. Add: what it does, how to install, how to run, how to upload, what the flags mean. With a terminal screenshot or example output.

---

*Analysis by simulated first-time user. 2026-03-27.*
