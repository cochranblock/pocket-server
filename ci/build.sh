#!/usr/bin/env bash
# Unlicense — cochranblock.org
# IRONHIVE CI — runs on push. Deploy to lf (n0) for lowest load.
# Pipeline: test → clippy → build release
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> [1/3] cargo test"
cargo test --locked 2>&1

echo "==> [2/3] cargo clippy"
cargo clippy --locked -- -D warnings 2>&1

echo "==> [3/3] cargo build --release"
cargo build --locked --release 2>&1

echo "==> CI passed"
