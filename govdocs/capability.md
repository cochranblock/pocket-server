# Capability Statement — Pocket Server

## Product

**Pocket Server** — a compiled Rust web server that runs on Android phones, serving static websites with zero cloud dependency.

## Vendor

**CochranBlock** — [cochranblock.org](https://cochranblock.org)
Zero-cloud architecture. 14 Unlicense repos. All products self-hosted on hardware the customer owns.

## Core Capabilities

- **Static web hosting** from local storage on Android devices
- **Live monitoring dashboard** with uptime, request count, bandwidth, power draw, cost estimation
- **File upload API** for remote site deployment (localhost-restricted)
- **Cloudflare tunnel integration** for public internet access through NAT/CGNAT
- **Foreground service** with wake lock for uninterrupted operation
- **Boot receiver** for automatic startup after device reboot

## Technical Specifications

- **Language:** Rust (compiled, memory-safe, no garbage collector)
- **Binary size:** ~1 MB (stripped, LTO, size-optimized)
- **Direct dependencies:** 3 (axum, tokio, tower-http)
- **Transport compression:** zstd
- **Architecture:** aarch64 (ARM64), armv7, iOS (staticlib)
- **Repository:** [github.com/cochranblock/pocket-server](https://github.com/cochranblock/pocket-server)
- **License:** Unlicense (public domain, no restrictions)

## Pricing Tiers

| Tier | Price | Includes |
|------|-------|----------|
| BYOD | $500 | Software license, configuration support |
| Starter | $750 | Refurbished phone, pre-configured |
| Turnkey | $1,000 | New phone + domain + ready to plug in |

## NAICS Codes

- 511210 — Software Publishers
- 541519 — Other Computer Related Services
- 334111 — Electronic Computer Manufacturing (hardware tiers)

## Past Performance

Part of the CochranBlock product line. Zero-cloud architecture deployed across multiple production systems.
