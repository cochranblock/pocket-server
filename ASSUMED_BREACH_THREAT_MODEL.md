# Assumed Breach Threat Model

> **Operating assumption: every component below is already compromised. Design for damage containment and loud detection, not for prevention.**

This document is the canonical threat model for every project in the `cochranblock/*` portfolio. Each project adapts the Threat Surface section for its own context but shares the same first principles, mitigations, and verification protocol.

---

## First Principles

1. **Every record that matters has an external witness.** Hashes published to public git (or equivalent neutral timestamp authority) so tampering requires simultaneously corrupting your system AND the public chain.
2. **No single point of compromise.** Signing keys in hardware (YubiKey / TPM / Secure Enclave). Never in software. Never in env vars. Never in config files.
3. **Default air-gap.** No network dependency for correctness. Network is for backup + publishing hashes, both signed, both verifiable post-hoc.
4. **Append-only everything.** No delete path in any storage layer. Corrections are reversing entries referencing the original. Standard accounting discipline, enforced in code.
5. **Cryptographic audit chain.** Every day's state derives from the previous day's hash. Tampering with any day invalidates every subsequent day.
6. **Disclosure of methodology is a security feature.** If an auditor can independently verify the algorithm, they can independently verify the outputs. No "trust us" layers.
7. **Separation of duties enforced in software.** Entry, approval, and audit live in different trust zones. Compromise of one does not compromise the others.
8. **Redundancy across trust zones.** Local + different-cloud + different-format + offline. Attacker must compromise all to hide damage.
9. **Test breach scenarios regularly.** Triple Sims applied to tamper detection. If the chain does not detect a simulated tamper, the chain is broken.

---

## Threat Surface (project-specific — adapt below)

Pocket Server is a compiled Rust web server for phones and desktops. It serves static files from `--site-dir`, exposes a live kiosk dashboard, and optionally spawns `cloudflared` as a child process to open a Cloudflare quick tunnel. It holds no signing keys, no user accounts, no persistent audit log, and no financial or legal records. Stats (`src/stats.rs`) are in-memory atomics that reset on every restart. The only persistent state is the directory of static files the user chose to serve, plus whatever they upload via the localhost-gated `/api/upload` endpoint.

This narrow scope means most of the "records of consequence" framework is N/A. What remains is a device-and-transport threat surface.

**Records this project emits:**

- **Release binary** (per-target: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `aarch64-linux-android` AAB, `aarch64-apple-ios` staticlib, Linux/Windows/FreeBSD/RISC-V/POWER). Integrity governed by `PROOF_OF_ARTIFACTS.md`, not this file.
- **Runtime-derived compliance endpoints**: `/govdocs/sbom` (SPDX 2.3 from `Cargo.lock`), `/govdocs/capability`, `/govdocs/security`, and the `--sbom` flag. These are pure functions of the binary; their integrity is the binary's integrity.
- **Uploaded files** placed under `--site-dir` via `/api/upload`. User-authored static-site content, not legal or audit records.
- **In-memory stats** (uptime, request count, bytes, power estimate, monthly cost) — volatile, non-authoritative, cosmetic.

**Applicable threats:**

- **Binary compromise on device** → attacker swaps `libpocket_server.so` (Android JNI), `libpocket_server.a` (iOS C FFI), or the native binary before launch. Serves attacker-chosen content under the user's site name. Mitigation: release-artifact signatures (see `PROOF_OF_ARTIFACTS.md`); on-device integrity is the OS's job.
- **Upload path traversal** → crafted filename with `..`, `\`, or absolute path tries to escape `--site-dir`. Mitigated in `src/server.rs` upload handler by splitting on both `/` and `\` and filtering out empty / `.` / `..` components; covered by `upload_path_traversal_blocked` and `upload_backslash_traversal_blocked` tests. Every new write path must re-verify this invariant.
- **Widening the write surface** → `/api/upload`, `/api/files/*` DELETE, and `/api/token` are gated by a localhost `ConnectInfo` check plus `X-Pocket-Token` CSRF header. Any future change that accepts a non-loopback `ConnectInfo` or drops the token check exposes remote write to the internet — especially with `--tunnel`. Treat those three handlers as the blast radius.
- **Cloudflare quick tunnel is not end-to-end** → `--tunnel` (`src/tunnel.rs`) spawns `cloudflared tunnel --url http://localhost:{port}`. TLS terminates at Cloudflare; the hop from cloudflared → localhost is plaintext HTTP (intentional — it rides the loopback interface). Anyone compromising the Cloudflare account, the `trycloudflare.com` subdomain, or the on-device `cloudflared` binary rides the wire.
- **Cloudflared child-process compromise** → pocket-server spawns an external binary it does not ship. If the on-device `cloudflared` is tampered, every tunneled request routes through attacker infrastructure before hitting the loopback server.
- **LAN exposure** → default bind is `0.0.0.0:{port}` (`src/server.rs` f9). On public Wi-Fi, neighbors can fetch the site and the dashboard. Upload / delete remain localhost-only, so this is read-only exposure, but the dashboard leaks request and byte counts.
- **Device seizure or loss** → phone contains the binary, the `--site-dir`, and uploaded content. Pocket Server enforces nothing here; full-disk encryption is the OS's job, and the hardware key (if any) is out of scope.
- **Supply chain (deps)** → three direct crates (`axum`, `tokio`, `tower-http`) plus `jni` on Android. Lean tree, but any upstream compromise lands in the release binary. Mitigated by `cargo audit` in CI, a pinned `Cargo.lock`, and the reproducible-build release profile (`opt-level=z`, `lto`, `codegen-units=1`, `strip`, `panic=abort`).
- **Static site tampering on disk** → attacker with device access rewrites files under `--site-dir`. Server has no content-integrity check — it serves whatever bytes are on disk. Out of scope for pocket-server; sites that need content provenance should sign their artifacts upstream.
- **Dashboard stats spoofing** → `src/stats.rs` atomics reset on restart and are never persisted. A compromised binary can report any numbers it likes. Cosmetic; no mitigation required because stats are not records of consequence.
- **Android service abuse** → the JNI boundary (`src/android.rs`, `PocketServer.startServer`) is a trust zone. If the Android manifest over-exports the service or intent filter, another app on the device could start/stop the server or read dashboard content.
- **Upload DoS** → without `--max-upload`, a multipart body could fill the device. Default cap is 50 MB with 413 rejection (recent commit `fd1f6d6`); this must stay on for untrusted networks even though upload is localhost-gated.

**N/A for this project (documented, not silently skipped):**

- **Hardware-key signing of runtime outputs** — no runtime outputs of consequence. Release-artifact signing lives in `PROOF_OF_ARTIFACTS.md`.
- **Daily hash chain / public-chain repo** — no audit records to hash. The Public-Chain Deployment section below is skipped per the Scope clause.
- **Append-only sled trees** — pocket-server does not use sled and persists no state beyond user-supplied files.
- **Separation-of-duties trust zones** — single-user, single-binary, single-device tool. No approval workflow because nothing requires approval.
- **Cryptographic audit chain / daily rollover** — no per-day records produced; uptime resets on every process restart.
- **Insider / self-tampering controls** — user owns the device, the binary, and the served files; there is no counterparty to protect the author from.
- **Clock manipulation controls** — stats are relative (uptime since `Instant::now()`); absolute wall-clock is not consulted for correctness.

If pocket-server's scope ever grows to include persistent request logs, auth records, or stateful audit artifacts, this section must be revisited and the public-chain deployment re-enabled.

---

## Mitigations

| Assume | Mitigation | Verification |
|--------|-----------|--------------|
| Binary compromised | Hardware-key signatures for every output of consequence | Anyone can verify the public key matches expected fingerprint |
| Storage compromised | Append-only sled trees. Delete is not a function, not a policy. | Hash chain breaks on any rewrite. External witness detects. |
| Network MITM | Air-gap capable. Network used only for signed backups + hash publishing. | NTP + GitHub timestamp + hardware counter cross-checked. |
| Signing key stolen | Daily hash committed to public git. Stolen key cannot retroactively change committed days. | Any day older than the public commit is immutable in evidence. |
| Audit log tampered | Separate sled tree, write-only from main app. Auditor tool reads both + cross-checks. | Compromise of main app leaves audit log intact. |
| Backup tampered | 3 different targets with 3 different credentials (local USB + off-site cloud + paper). | Attacker needs all three to hide damage. |
| Insider / self-tampering | No admin role. No delete. Reversing entries only. | Legal record immune to author second-thoughts. |
| Clock manipulation | Multiple time sources: local clock, NTP, git commit timestamp, hardware-key counter. | Divergence flags exception requiring supervisor approval. |
| Supply chain (deps) | `cargo audit` in CI. Pinned SBOM. Reproducible builds where possible. | Anyone can reproduce the binary from source + lockfile. |
| Physical device seizure | Full-disk encryption. Hardware key physically separate from device. | Stolen laptop without key is useless for forgery. |

---

## Public-Chain Deployment

This project publishes tamper-evident hashes to a public companion repo: `cochranblock/<project>-chain` (where `<project>` is the project name).

- **Daily cycle:** at 23:59 local, compute BLAKE3 of all records-of-consequence from the day. Sign with hardware key. Commit to chain repo. Push.
- **GitHub timestamp** on the commit = neutral third-party witness. Anyone can cold-verify records were not rewritten after commit time.
- **Verification:** `<project> verify` reads the chain and re-derives hashes. Any divergence = tampering detected.

This pattern is a private Certificate Transparency log for project state. Same primitive Google uses for TLS certs, applied to whatever the project tracks.

---

## Triple Sims for Tamper Detection

Standard Triple Sims gate (run 3x identically) extended with a tamper-scenario sim:

1. Normal run → produce canonical output
2. Simulated tampering (flip one bit in storage) → `verify` must flag it
3. Simulated clock rewind → `verify` must flag it

If any sim fails to detect, the chain is broken. Fix before merge.

---

## Scope of this Document

- Covers: any artifact this project emits that has legal, financial, or audit consequence.
- Does NOT cover: source code itself (public under Unlicense, not sensitive), build outputs (reproducible), marketing content (public by design).
- If your project emits no records of consequence, the relevant sections are zero-length and the public-chain deployment is skipped. Document that explicitly.

---

## Relation to Other Docs

- **TIMELINE_OF_INVENTION.md** — establishes priority dates for contributions. Feeds into the chain's initial state.
- **PROOF_OF_ARTIFACTS.md** — cryptographic signatures on release artifacts. Adjacent pattern, same first principles.
- **DCAA_COMPLIANCE.md** (where applicable) — how this threat model satisfies FAR/DFARS audit requirements.

---

## Status

- [ ] Threat Surface section adapted for this project
- [ ] Hardware-key signing integrated or N/A documented
- [ ] Public-chain repo created and connected or N/A documented
- [ ] Triple Sims tamper-detection test present or N/A documented
- [ ] External verification procedure documented

---

*Unlicensed. Public domain. Fork, strip attribution, adapt, ship.*

*Canonical source: cochranblock.org/threat-model — last revision 2026-04-14*
