# Operator Setup Guide — Pocket POS

A walkthrough for new operators getting Pocket POS up and taking taps. ~10 minutes if you already have a Stripe account; ~20 minutes if not.

## What you need

- Any NFC-equipped Android phone or tablet running Android 13+ (Tap to Pay won't work on older Android). For the SDK device matrix see `docs/stripe-device-matrix.md`.
- A Stripe account ([stripe.com](https://stripe.com) — free, no monthly fee).
- 5–10 minutes.

---

## Step 1 — Create or sign in to Stripe

Go to [stripe.com](https://stripe.com) and sign up. Stripe will collect:

- Your business name, address, and EIN/SSN (for tax reporting; Stripe is the merchant of record)
- Your bank account routing + account number (for payouts)

You can do all of this in test mode first if you want to play before processing real money — test mode requires no bank details. Just a name + email.

## Step 2 — Get your secret key

In the Stripe dashboard:

1. Click **Developers → API keys** (or open [dashboard.stripe.com/apikeys](https://dashboard.stripe.com/apikeys)).
2. Toggle **Test mode** in the top-right (until you're ready for real charges).
3. Reveal the **Secret key** — it starts with `sk_test_…` or `sk_live_…`.
4. Copy it to the clipboard. **This key has full access to your Stripe account — treat it like a password.**

> **Live keys are scary on purpose.** Use `sk_test_…` while you're learning. Switch to `sk_live_…` only when you're ready to process real cards.

## Step 3 — Register a Location

Stripe Tap to Pay requires a **Location** (a Stripe-side identifier representing your physical operating address).

1. In the Stripe dashboard, go to [dashboard.stripe.com/terminal/locations](https://dashboard.stripe.com/terminal/locations).
2. Click **+ Create location**.
3. Enter your business address (food truck home address, market stall location, etc.).
4. Save. The new location has an id like `tml_abc123` — keep it open in another tab.

You'll need both the secret key and the location id in the next step.

## Step 4 — Configure Pocket POS

You have two options:

### Option A — Edit the config file directly

Find or create `~/.pocket-server.toml` on the device running the server. Add:

```toml
name = "My Food Truck"
port = 8080
stripe_secret_key = "sk_test_paste_yours_here"
stripe_location_id = "tml_paste_yours_here"
```

### Option B — Use the in-app Setup screen *(once Phase A Android module ships)*

When you first launch Pocket POS on Android, a Setup screen will guide you:

1. Paste your secret key from Step 2.
2. Pick your Location from the auto-populated list (calls `GET /v1/terminal/locations` for you).
3. Tap **Test connection** — verifies Stripe is reachable.
4. Tap **Save** — writes to `~/.pocket-server.toml` and restarts the server.

Either way, the secret key never leaves the device. The native app reads it via JNI, the Stripe SDK uses it to authenticate. No web traffic.

## Step 5 — Enable receipt emails (optional, recommended)

Stripe can send a receipt email to the customer's address automatically — no work for you.

1. Go to [dashboard.stripe.com/settings/customer-emails](https://dashboard.stripe.com/settings/customer-emails) (or **Settings → Customer emails**).
2. Toggle **Receipts** to **On**.
3. Customize the receipt logo / colors if you want.

When you collect a card via Tap to Pay, Stripe asks the customer if they want a receipt — they enter their email at the moment of tap (Apple Pay / Google Pay tokenized cards include the email automatically).

## Step 6 — Verify the wire

Start the server and check the banner:

```
$ ./pocket-server --port 8080
pocket-server v0.1.0
  name:     My Food Truck
  port:     8080
  ...
  stripe:    connected (test mode)         ← look for this!
```

If it says `not configured`, your `stripe_secret_key` either wasn't found or isn't a recognized format. Check the spelling and that the file is at `~/.pocket-server.toml`.

You can also hit `http://localhost:8080/api/stripe/status` (loopback only):

```json
{"connected": true, "mode": "test"}
```

## Step 7 — Take your first test tap

Once the Pocket POS Android module is installed (Phase A — currently in development):

1. Open the Pocket POS app on your phone/tablet.
2. The Setup screen detects your config and confirms `stripe: connected (test mode)`.
3. Either type an amount manually or use the menu page (drop your `index.html` from `docs/templates/menu.html` into your `site_dir`).
4. Tap **Charge** on a pending order.
5. Stripe Terminal SDK shows the "Hold card near phone" prompt.
6. Use a [Stripe test card](https://docs.stripe.com/testing) (e.g., `4242 4242 4242 4242` via Apple Pay / Google Pay) to simulate a tap.
7. The order flips to **charged**, the receipt screen appears, the order appears in your Stripe dashboard.

## Step 8 — Switch to live mode

When you've tested and you're ready for real money:

1. In Stripe dashboard, toggle **Test mode** off.
2. Copy the new `sk_live_…` key.
3. Replace `stripe_secret_key` in `~/.pocket-server.toml`.
4. Replace `stripe_location_id` if Stripe issued a different live-mode location id (sometimes they're the same, sometimes not).
5. Restart pocket-server. Banner should now read `CONNECTED (LIVE MODE)` in red caps.

**Live mode means real cards, real charges, real money.** Tap-to-Pay test cards do not work in live mode. You'll need a real card from a real customer.

## Where your data lives

After running for a while, you'll have:

- `~/.pocket-server.toml` — your config (incl. Stripe secret key)
- `~/.pocket-server-orders.jsonl` — append-only log of every order ever
- `<site_dir>/...` — your menu / website files

Everything is on **your device**. To export it all (e.g., to migrate, back up, or hand to your accountant), hit:

```
curl http://localhost:8080/api/export/all > pocket-pos-export.tar
```

You'll get a tarball with orders, config (key redacted), and your site files. Extract with `tar -xf`. Yours, forever.

## Common issues

- **"stripe: not configured"** — `~/.pocket-server.toml` missing or `stripe_secret_key` line missing/typo.
- **Tap to Pay won't initialize on Android** — confirm Android 13+ (`adb shell getprop ro.build.version.sdk` ≥ 33), NFC enabled in system settings, device not rooted.
- **"No locations found"** in Setup — create one at [dashboard.stripe.com/terminal/locations](https://dashboard.stripe.com/terminal/locations) first.
- **Card declined in test mode** — test mode declines real cards; use a [Stripe test card number](https://docs.stripe.com/testing).
- **Card declined in live mode** — could be customer's bank, a flagged transaction, or a Stripe risk rule. Customer should try a different card.

## Refunds

Pocket POS does not implement refunds in v1. To refund a customer:

1. Open [dashboard.stripe.com/payments](https://dashboard.stripe.com/payments).
2. Find the charge.
3. Click **Refund**, choose full or partial.
4. Stripe handles the rest.

This is intentional — refunds touch real money flow, deserve their own UX, and are rare enough that the Stripe dashboard is fine for v1.

## Pricing reminder

- **Pocket POS license:** Free, public domain (Unlicense). No setup fee, no monthly fee, no per-transaction markup.
- **Stripe US Tap to Pay rate:** 2.7% + $0.15 per transaction (2.7% + $0.05 base + $0.10 Tap to Pay surcharge). [stripe.com/pricing](https://stripe.com/pricing).
- **International cards:** +1.5% on top.
- **Currency conversion:** +1% on top if applicable.

Your operating cost on a $12 ticket is about **$0.47** total (3.92%) — paid only to Stripe, none to us.

---

For the full data-flow / sovereignty story (especially if procurement is asking) see `govdocs/deployment.md`.
For the supported device list see `docs/stripe-device-matrix.md`.
<!-- COCHRANBLOCK-BRAND-FOOTER:START - generated by cochranblock/scripts/brand-stamp.sh -->

---

<sub>&#9656; **THE COCHRAN BLOCK, LLC** &#183; CAGE `1CQ66` &#183; UEI `W7X3HAQL9CF9` &#183; UNLICENSE &#183; [cochranblock.org](https://cochranblock.org)</sub>
<!-- COCHRANBLOCK-BRAND-FOOTER:END -->
