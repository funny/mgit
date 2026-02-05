#!/bin/bash
set -e

VERSION=$1
if [ -z "$VERSION" ]; then
    VERSION="0.0.0"
fi

APP_NAME="Mgit"
APP_DIR="$APP_NAME.app"
BINARY_DIR="target/release"
RESOURCE_DIR="mgit-gui/resource"
PACKAGING_DIR="scripts/macos"
OUTPUT_DMG="$2"

if [ -z "$OUTPUT_DMG" ]; then
    OUTPUT_DMG="$PACKAGING_DIR/mgit.dmg"
fi

# Clean up
rm -rf "$APP_DIR" "$OUTPUT_DMG"

# Create directory structure
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

# Copy binaries
echo "Copying binaries..."
if [ -f "$BINARY_DIR/mgit-gui" ]; then
    cp "$BINARY_DIR/mgit-gui" "$APP_DIR/Contents/MacOS/"
else
    echo "Error: mgit-gui not found in $BINARY_DIR"
    exit 1
fi

if [ -f "$BINARY_DIR/mgit" ]; then
    cp "$BINARY_DIR/mgit" "$APP_DIR/Contents/MacOS/"
else
    echo "Warning: mgit cli not found in $BINARY_DIR"
fi

# Copy Info.plist
cp "$PACKAGING_DIR/Info.plist" "$APP_DIR/Contents/Info.plist"
# Replace version
sed -i '' "s/__VERSION__/$VERSION/g" "$APP_DIR/Contents/Info.plist"

# Generate ICNS
echo "Generating icon..."
mkdir -p Mgit.iconset
sips -z 16 16     "$RESOURCE_DIR/logo128x128.png" --out Mgit.iconset/icon_16x16.png
sips -z 32 32     "$RESOURCE_DIR/logo128x128.png" --out Mgit.iconset/icon_16x16@2x.png
sips -z 32 32     "$RESOURCE_DIR/logo128x128.png" --out Mgit.iconset/icon_32x32.png
sips -z 64 64     "$RESOURCE_DIR/logo128x128.png" --out Mgit.iconset/icon_32x32@2x.png
sips -z 128 128   "$RESOURCE_DIR/logo128x128.png" --out Mgit.iconset/icon_128x128.png
sips -z 256 256   "$RESOURCE_DIR/logo128x128.png" --out Mgit.iconset/icon_128x128@2x.png
iconutil -c icns Mgit.iconset
cp Mgit.icns "$APP_DIR/Contents/Resources/AppIcon.icns"
rm -rf Mgit.iconset Mgit.icns

# Sign (Ad-hoc)
echo "Signing..."
codesign --sign - --force --deep --options runtime "$APP_DIR"

# Create DMG
echo "Creating DMG..."
# Check if create-dmg exists
if ! command -v create-dmg &> /dev/null; then
    echo "create-dmg could not be found"
    exit 1
fi

create-dmg \
  --volname "MGIT Installer" \
  --volicon "$APP_DIR/Contents/Resources/AppIcon.icns" \
  --window-pos 200 120 \
  --window-size 800 400 \
  --icon-size 100 \
  --icon "$APP_NAME.app" 200 190 \
  --hide-extension "$APP_NAME.app" \
  --app-drop-link 600 185 \
  "$OUTPUT_DMG" \
  "$APP_DIR"
