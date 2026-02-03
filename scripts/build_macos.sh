#!/bin/sh
set -e

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist/macos"

mkdir -p "$DIST_DIR"

cd "$ROOT_DIR"
cargo build --release -p aegis-core-server -p aegis-collector-cli -p aegis-verifier

cp "target/release/aegis-core-server" "$DIST_DIR/"
cp "target/release/aegis-collector-cli" "$DIST_DIR/"
cp "target/release/aegis-verifier" "$DIST_DIR/"

if command -v swift >/dev/null 2>&1; then
  cd "$ROOT_DIR/collectors/macos/native_recorder"
  swift build -c release
  cp ".build/release/aegis-native-recorder" "$DIST_DIR/"
fi

echo "Artifacts in $DIST_DIR"
