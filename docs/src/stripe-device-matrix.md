<!-- Unlicense — cochranblock.org -->

# Stripe Terminal Tap to Pay — Device & Region Matrix

Research artifact for backlog #2. Source-of-truth for what Pocket POS can promise about device support, regions, and economics. **Verify before quoting in marketing copy or capability statements** — Stripe's regional and device lists move quarterly.

> **As-of-research:** 2026-05-06. Re-validate every quarter; flag anything older than 90 days as stale.

## Bottom-line for Pocket POS

| Question | Answer |
|----------|--------|
| Android phones with NFC | **Supported.** Android 13+ at runtime, ARM, unrooted, locked bootloader, security patch ≤ 12 mo old. |
| Android tablets with NFC | **Explicitly supported with named device list.** See § Devices. |
| iPhone | **Supported.** iPhone XS or later, iOS varies by region (16.7+ US/AU/UK, 17.0+ FR/IT/NL, 17.4+ CA). |
| iPad | **NOT supported by Stripe Terminal SDK.** iPhone-only on Apple side. |
| Stripe Terminal Android SDK install minimum | API 26 (Android 8.0). Tap to Pay feature requires API 33 / Android 13 at *runtime*, but the SDK installs on older devices. |
| Stripe Terminal Android SDK current version | 5.5.0 (as of 2026-05-06). |
| Tap to Pay regions (Stripe Android) | 34+ countries. |
| Tap to Pay regions (Stripe iPhone) | 36+ countries. |
| Tap to Pay regions (Apple — iPhone) | 50+ countries (Apple's gate is broader than Stripe's). |
| US per-transaction fee | **2.7% + $0.05 base + $0.10 Tap to Pay surcharge = 2.7% + $0.15 per tap.** |
| International card surcharge | +1.5%. |
| Currency conversion | +1%. |

## Implications for our build

### Architecture impact

- **Pocket-server's Android module already targets API 26 — matches Stripe SDK install minimum.** No bump required.
- **Tap to Pay is a runtime feature gate.** Operator phones running Android 8–12 can still run pocket-server with manual order entry, just no Tap to Pay flow. Document this clearly in the operator UX.
- **AAB size will jump significantly** when we add the SDK. Current AAB is 721 KB; expect 5–15 MB after `stripeterminal-core` + `stripeterminal-taptopay`. The "1.5 MB Rust binary" pitch breaks for the POS variant — lead instead with "single-app POS, no hardware."
- **iPad is out.** Earlier internal claim that iPad Pro M-series might work via Stripe is incorrect — Stripe Terminal SDK is iPhone-only on Apple. If we want iPad someday, we'd have to integrate Apple's PassKit/ProximityReader directly, bypassing Stripe. That's a separate, much larger workstream.

### Pricing copy correction

Earlier internal estimate ("2.7% + $0.05") was for general Stripe Terminal in-person. **Tap to Pay carries an additional $0.10 per authorization, US-wide.** Correct figure: **2.7% + $0.15** per Tap to Pay transaction. Keep this exact in any operator-facing materials.

### Gradle dependencies (Android module, when we get there)

```kotlin
dependencies {
    implementation("com.stripe:stripeterminal-core:5.5.0")
    implementation("com.stripe:stripeterminal-taptopay:5.5.0")
}
```

Java 8 source/target compatibility required.

## Devices

### Android phones — class of supported

Any Android device meeting all of:
- NFC sensor (integrated, functioning)
- ARM-based processor
- Android **13 or later** (API 33+)
- Hardware keystore v100+
- Unrooted, bootloader locked & unchanged
- Google Mobile Services + Play Store installed
- Security update ≤ 12 months old

Practical: most flagship and mid-range NFC Android phones from 2022 onward will work. Kindle Fire, Huawei (no Google services), de-Googled phones (GrapheneOS, /e/) **will not**.

### Android tablets — explicit device list (Stripe docs)

Per docs.stripe.com/terminal/payments/setup-reader/tap-to-pay (canonical):

| Tablet | Class | Notes |
|--------|-------|-------|
| Sunmi CPad | Rugged commercial POS | Common in retail/QSR; Sunmi makes Stripe-certified hardware. |
| Samsung Galaxy Tab Active5 | Rugged consumer | Mainstream, easy to source. ~$650 retail. |
| HMD Global HMD T21 | Mid-tier consumer | Cheaper option. |
| Oukitel RT3 Plus | Rugged consumer | Budget rugged. |
| Ulefone Armor Pad 4 Ultra | Rugged consumer | Budget rugged. |

**Standard Galaxy Tab S series (S7/S8/S9/S10) are NOT on Stripe's named list.** They have NFC and run Android 13+, so per the general Android requirements they should work — but they're not certified by Stripe. **Recommend testing with one before promising it to operators.**

### Apple — iPhone only

Stripe Terminal SDK supports iPhone XS or later. iOS minimum varies by region. See § Regions.

iPad (any model, including NFC-equipped iPad Pro M-series) is **not supported** by Stripe Terminal Tap to Pay. Apple's own Tap to Pay framework may extend to iPad in iPadOS, but Stripe's SDK does not surface that.

## Regions

### Stripe Tap to Pay on iPhone (subset of Apple's list — Stripe gates further)

US, UK, Australia, Canada, Ireland, Italy, Netherlands, Austria, Czech Republic, France, Germany, New Zealand, Sweden, plus more — **36+ total**. Confirm exact current list at docs.stripe.com/terminal/payments/regional before committing in marketing.

### Stripe Tap to Pay on Android

US, Canada, UK, Ireland, New Zealand, Singapore, Australia, Austria, Belgium, Czech Republic, Denmark, Finland, France, Germany, Italy, Luxembourg, Malaysia, Netherlands, Norway, Poland, Portugal, Spain, Sweden, Switzerland — **24+ countries** (sources vary 24–34, recheck before quoting).

### Apple's broader Tap to Pay region list (informational, not Stripe-gated)

50+ countries: all of Western + most of Eastern Europe, Brazil, Chile, Mexico, Hong Kong, Japan, Malaysia, Taiwan, UAE. Of operational interest: even where Apple has enabled Tap to Pay on iPhone, **Stripe must also be available in that market** — they're independent gates.

## Regional payment-method notes

- **Australia:** eftpos supported via Tap to Pay.
- **Canada:** Interac supported. Caveat: many Canadian-issued cards are offline-PIN-only, which Tap to Pay cannot collect — those cards must use a Bluetooth/USB reader instead.
- **France:** Cartes Bancaires supported.
- **Finland:** Same offline-PIN issue as Canada.
- **UK, Canada, Finland:** Regional issuer policies can affect contactless transaction limits — Tap to Pay may downgrade to chip-and-PIN for above-threshold amounts, which the operator would need a different reader to handle.

## Account requirements

- Operator needs a Stripe account (Standard or Connect Express). For pocket-server's self-hosted, $0-platform-fee model: **operator creates a Standard Stripe account themselves**, supplies their own `sk_test_…` / `sk_live_…` to `~/.pocket-server.toml`. No platform middleman.
- Stripe requires a `Location` (a Connect/Account-side identifier) per physical operating address. For a food truck, this is set once during onboarding. Stripe Terminal SDK requires the location_id on every reader connection.
- Internet connection required at all times — Tap to Pay does not store offline transactions. (Bluetooth Stripe readers do, in case operators need offline fallback.)

## Open questions / verify before launch

1. **Galaxy Tab S series uncertified status** — Stripe's named tablets are all rugged commercial. Standard consumer tablets (Galaxy Tab S9, Lenovo Tab P12) likely work via the general Android criteria but aren't blessed. Procure one and test before promising support.
2. **Beta status of Tap to Pay on Android** — some third-party sources still describe it as "beta." Stripe's main docs do not. Confirm GA status with Stripe support before marketing it as production-ready.
3. **Region accuracy** — Stripe's regional availability list churns quarterly. Pin a date when you check it (this doc: 2026-05-06).
4. **Connect Express vs Standard** — for the self-hosted MVP, Standard is cleaner (operator owns their account directly). If we ever offer a hosted onboarding tier later, Connect Express becomes relevant. Don't engineer for Connect until we decide.
5. **iPad alternative path** — if iPad operators are a meaningful segment, evaluate Apple's PassKit ProximityReader API as a non-Stripe path. Engineering cost is substantially higher (Stripe abstracts a lot).

## Sources

- Stripe Tap to Pay overview: https://stripe.com/terminal/tap-to-pay
- Stripe Tap to Pay docs (canonical): https://docs.stripe.com/terminal/payments/setup-reader/tap-to-pay
- Stripe Tap to Pay on iPhone: https://stripe.com/terminal/tap-to-pay-on-iphone
- Stripe regional considerations: https://docs.stripe.com/terminal/payments/regional
- Stripe Terminal Android SDK: https://github.com/stripe/stripe-terminal-android
- Apple Tap to Pay on iPhone region list: https://developer.apple.com/tap-to-pay/regions/
- Stripe pricing: https://stripe.com/pricing
- Stripe Tap to Pay launch announcement (Android): https://stripe.com/newsroom/news/tap-to-pay-android
<!-- COCHRANBLOCK-BRAND-FOOTER:START - generated by cochranblock/scripts/brand-stamp.sh -->

---

<sub>&#9656; **THE COCHRAN BLOCK, LLC** &#183; CAGE `1CQ66` &#183; UEI `W7X3HAQL9CF9` &#183; UNLICENSE &#183; [cochranblock.org](https://cochranblock.org)</sub>
<!-- COCHRANBLOCK-BRAND-FOOTER:END -->
