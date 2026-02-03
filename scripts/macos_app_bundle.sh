#!/bin/sh
set -e

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist/macos"
APP_NAME="AEGISTRACE"
APP_DIR="$DIST_DIR/${APP_NAME}.app"

mkdir -p "$DIST_DIR"

"$ROOT_DIR/scripts/build_macos.sh"

rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

cat > "$APP_DIR/Contents/Info.plist" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>AEGISTRACE</string>
  <key>CFBundleIdentifier</key>
  <string>com.aegistrace.app</string>
  <key>CFBundleVersion</key>
  <string>0.1.0</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.0</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleExecutable</key>
  <string>AEGISTRACE</string>
</dict>
</plist>
EOF

cat > "$APP_DIR/Contents/MacOS/AEGISTRACE" <<'EOF'
#!/bin/sh
set -e

APP_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BIN_DIR="$APP_DIR/MacOS"

echo "AEGISTRACE app bundle"
echo "Binaries are in: $BIN_DIR"
echo "Example:"
echo "  $BIN_DIR/aegis-core-server macos 0.1.0"
echo "  $BIN_DIR/aegis-collector-cli focus com.apple.finder Finder"
echo "  $BIN_DIR/aegis-verifier verify /path/to/Evidence_..."
EOF

chmod +x "$APP_DIR/Contents/MacOS/AEGISTRACE"

cp "$DIST_DIR/aegis-core-server" "$APP_DIR/Contents/MacOS/"
cp "$DIST_DIR/aegis-collector-cli" "$APP_DIR/Contents/MacOS/"
cp "$DIST_DIR/aegis-verifier" "$APP_DIR/Contents/MacOS/"
if [ -f "$DIST_DIR/aegis-native-recorder" ]; then
  cp "$DIST_DIR/aegis-native-recorder" "$APP_DIR/Contents/MacOS/"
fi

echo "App bundle created at $APP_DIR"
