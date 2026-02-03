#!/bin/sh
set -e

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
RECORDER_DIR="$ROOT_DIR/collectors/macos/native_recorder"
OUTPUT_PATH="${1:-/tmp/aegis_screen.mp4}"
SECONDS="${2:-3}"

cd "$RECORDER_DIR"

if [ ! -x ".build/release/aegis-native-recorder" ]; then
  swift build -c release
fi

./.build/release/aegis-native-recorder "$OUTPUT_PATH" "$SECONDS"
