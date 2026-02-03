#!/bin/sh
set -e

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"

PLATFORM="macos"
APP_VERSION="0.1.0"

cd "$ROOT_DIR"

cargo run -p aegis-core-server -- "$PLATFORM" "$APP_VERSION" &
SERVER_PID=$!

sleep 1

TMP_SCREEN="/tmp/aegis_screen.mp4"
if [ -x "$ROOT_DIR/collectors/macos/run_native_recorder.sh" ] && command -v swift >/dev/null 2>&1; then
  "$ROOT_DIR/collectors/macos/run_native_recorder.sh" "$TMP_SCREEN" "3"
elif command -v ffmpeg >/dev/null 2>&1; then
  ffmpeg -y -f lavfi -i color=c=black:s=640x360:d=1 -pix_fmt yuv420p "$TMP_SCREEN" >/dev/null 2>&1
else
  printf "AEGIS DEMO SCREEN" > "$TMP_SCREEN"
fi

cargo run -p aegis-collector-cli -- focus "com.apple.finder" "Finder" "Desktop"
cargo run -p aegis-collector-cli -- file "$TMP_SCREEN" "files/screen.mp4" "screen_recording"
TMP_SHOT="/tmp/aegis_shot.jpg"
if command -v screencapture >/dev/null 2>&1; then
  screencapture -x -t jpg "$TMP_SHOT"
else
  printf "AEGIS DEMO SHOT" > "$TMP_SHOT"
fi
cargo run -p aegis-collector-cli -- shot "$TMP_SHOT" "files/shots/000001.jpg"
cargo run -p aegis-collector-cli -- input "10000" "42" "3" "1"
cargo run -p aegis-collector-cli -- stop "demo"

wait "$SERVER_PID"
