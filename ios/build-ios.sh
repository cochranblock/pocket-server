#!/bin/bash
# Unlicense — cochranblock.org
# Build pocket-server static library for iOS, then Xcode archive.
# Run from project root: ./ios/build-ios.sh

set -euo pipefail

PROJ_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJ_ROOT"

echo "=== Building Rust staticlib for aarch64-apple-ios ==="
cargo build --release --target aarch64-apple-ios --lib

LIB="$PROJ_ROOT/target/aarch64-apple-ios/release/libpocket_server.a"
if [ ! -f "$LIB" ]; then
    echo "error: staticlib not found at $LIB"
    exit 1
fi
echo "  staticlib: $(wc -c < "$LIB") bytes"

echo ""
echo "=== Rust library built ==="
echo "  $LIB"
echo ""
echo "Next steps (interactive — run in Xcode or terminal):"
echo "  1. Create Xcode project targeting ios/PocketServer/"
echo "  2. Link $LIB as a static library"
echo "  3. Add ios/PocketServer/AppDelegate.swift as source"
echo "  4. Set Info.plist to ios/PocketServer/Info.plist"
echo "  5. Add ios/Assets.xcassets for app icon"
echo "  6. Build: xcodebuild -scheme PocketServer -sdk iphoneos archive"
echo ""
echo "Or use cargo-xcode to generate the .xcodeproj automatically:"
echo "  cargo install cargo-xcode"
echo "  cargo xcode"
