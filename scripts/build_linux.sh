#!/bin/sh
set -e

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist/linux"

mkdir -p "$DIST_DIR"

cd "$ROOT_DIR"
cargo build --release -p aegis-core-server -p aegis-collector-cli -p aegis-verifier

cp "target/release/aegis-core-server" "$DIST_DIR/"
cp "target/release/aegis-collector-cli" "$DIST_DIR/"
cp "target/release/aegis-verifier" "$DIST_DIR/"

echo "Artifacts in $DIST_DIR"
