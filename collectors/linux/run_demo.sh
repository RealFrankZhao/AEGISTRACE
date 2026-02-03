#!/bin/sh
set -e

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"

PLATFORM="linux"
APP_VERSION="0.1.0"

cd "$ROOT_DIR"

cargo run -p aegis-core-server -- "$PLATFORM" "$APP_VERSION" &
SERVER_PID=$!

sleep 1

TMP_SCREEN="/tmp/aegis_screen.mp4"
printf "AEGIS DEMO SCREEN" > "$TMP_SCREEN"

cargo run -p aegis-collector-cli -- focus "org.gnome.Terminal" "Terminal" "Shell"
cargo run -p aegis-collector-cli -- file "$TMP_SCREEN" "files/screen.mp4" "screen_recording"
TMP_SHOT="/tmp/aegis_shot.jpg"
printf "AEGIS DEMO SHOT" > "$TMP_SHOT"
cargo run -p aegis-collector-cli -- shot "$TMP_SHOT" "files/shots/000001.jpg"
cargo run -p aegis-collector-cli -- input "10000" "42" "3" "1"
cargo run -p aegis-collector-cli -- stop "demo"

wait "$SERVER_PID"
