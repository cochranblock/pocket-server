#!/bin/bash
# Unlicense — cochranblock.org
# Build pocket-server for every supported target.
# Uses `cross` for Linux/Windows/FreeBSD/RISC-V/POWER targets.
# Run from project root: ./build-all-targets.sh
#
# Prerequisites:
#   cargo install cross
#   rustup target add aarch64-apple-darwin x86_64-apple-darwin aarch64-apple-ios wasm32-unknown-unknown

set -euo pipefail

PROJECT="pocket-server"
VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
RELEASE_DIR="release"
mkdir -p "$RELEASE_DIR"

echo "=== Building $PROJECT v$VERSION for all targets ==="
echo ""

# Track successes
BUILT=()
FAILED=()

build_native() {
    local target=$1
    echo "--- $target (native) ---"
    if cargo build --release --target "$target" 2>&1; then
        local ext=""
        [[ "$target" == *"windows"* ]] && ext=".exe"
        local src="target/$target/release/${PROJECT}${ext}"
        local dst="$RELEASE_DIR/${PROJECT}-${target}${ext}"
        if [ -f "$src" ]; then
            cp "$src" "$dst"
            echo "  -> $dst ($(wc -c < "$dst") bytes)"
            BUILT+=("$target")
        else
            echo "  ! binary not found at $src"
            FAILED+=("$target")
        fi
    else
        echo "  ! build failed"
        FAILED+=("$target")
    fi
    echo ""
}

build_cross() {
    local target=$1
    echo "--- $target (cross) ---"
    if cross build --release --target "$target" 2>&1; then
        local ext=""
        [[ "$target" == *"windows"* ]] && ext=".exe"
        local src="target/$target/release/${PROJECT}${ext}"
        local dst="$RELEASE_DIR/${PROJECT}-${target}${ext}"
        if [ -f "$src" ]; then
            cp "$src" "$dst"
            echo "  -> $dst ($(wc -c < "$dst") bytes)"
            BUILT+=("$target")
        else
            echo "  ! binary not found at $src"
            FAILED+=("$target")
        fi
    else
        echo "  ! build failed"
        FAILED+=("$target")
    fi
    echo ""
}

build_lib() {
    local target=$1
    local desc=$2
    echo "--- $target ($desc) ---"
    if cargo build --release --target "$target" --lib 2>&1; then
        echo "  -> lib built for $target"
        BUILT+=("$target")
    else
        echo "  ! build failed"
        FAILED+=("$target")
    fi
    echo ""
}

# ---- macOS (native, runs on this machine) ----
build_native "aarch64-apple-darwin"
build_native "x86_64-apple-darwin"

# ---- iOS (staticlib only, no binary) ----
build_lib "aarch64-apple-ios" "staticlib"

# ---- WASM (lib only) ----
build_lib "wasm32-unknown-unknown" "wasm"

# ---- Linux (cross-compiled via `cross` or build on remote) ----
if command -v cross &>/dev/null; then
    build_cross "x86_64-unknown-linux-gnu"
    build_cross "aarch64-unknown-linux-gnu"
    build_cross "armv7-unknown-linux-gnueabihf"
    build_cross "riscv64gc-unknown-linux-gnu"
    build_cross "x86_64-unknown-freebsd"
    build_cross "powerpc64le-unknown-linux-gnu"
    build_cross "x86_64-pc-windows-gnu"
else
    echo "=== cross not installed — skipping Linux/Windows/FreeBSD targets ==="
    echo "  Install: cargo install cross"
    echo "  Or build on target machines (rsync + cargo build --release)"
    echo ""
fi

# ---- Android (cdylib via cargo-ndk) ----
if command -v cargo-ndk &>/dev/null; then
    echo "--- aarch64-linux-android (cargo-ndk) ---"
    if cargo ndk --target aarch64-linux-android --platform 26 -- build --release 2>&1; then
        local_so="target/aarch64-linux-android/release/libpocket_server.so"
        if [ -f "$local_so" ]; then
            cp "$local_so" "$RELEASE_DIR/libpocket_server-aarch64-android.so"
            echo "  -> $RELEASE_DIR/libpocket_server-aarch64-android.so"
            BUILT+=("aarch64-linux-android")
        fi
    fi
    echo ""
else
    echo "=== cargo-ndk not installed — skipping Android target ==="
    echo ""
fi

# ---- Summary ----
echo "============================================"
echo "  BUILT: ${#BUILT[@]} targets"
for t in "${BUILT[@]}"; do echo "    + $t"; done
if [ ${#FAILED[@]} -gt 0 ]; then
    echo "  FAILED: ${#FAILED[@]} targets"
    for t in "${FAILED[@]}"; do echo "    - $t"; done
fi
echo "============================================"
echo ""
ls -lh "$RELEASE_DIR/"
