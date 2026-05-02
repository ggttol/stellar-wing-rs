#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
APP_NAME="Stellar Wing"
BIN_NAME="stellar-wing"
BUILD_DIR="$ROOT_DIR/target/release"
DIST_DIR="$ROOT_DIR/dist"
APP_DIR="$DIST_DIR/$APP_NAME.app"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"
ICON_SRC="$ROOT_DIR/assets/app_icon.png"
ICONSET_DIR="$DIST_DIR/AppIcon.iconset"
ICON_ICNS="$RESOURCES_DIR/AppIcon.icns"

mkdir -p "$DIST_DIR"

cd "$ROOT_DIR"

# 从 Cargo.toml 抽出版本号塞进 Info.plist；保持 .app 的 CFBundleVersion
# 始终与发布二进制一致。
APP_VERSION=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -1)

cargo build --release

rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR" "$RESOURCES_DIR"

if [[ -f "$ICON_SRC" ]]; then
    rm -rf "$ICONSET_DIR"
    mkdir -p "$ICONSET_DIR"

    sips -z 16 16 "$ICON_SRC" --out "$ICONSET_DIR/icon_16x16.png" >/dev/null
    sips -z 32 32 "$ICON_SRC" --out "$ICONSET_DIR/icon_16x16@2x.png" >/dev/null
    sips -z 32 32 "$ICON_SRC" --out "$ICONSET_DIR/icon_32x32.png" >/dev/null
    sips -z 64 64 "$ICON_SRC" --out "$ICONSET_DIR/icon_32x32@2x.png" >/dev/null
    sips -z 128 128 "$ICON_SRC" --out "$ICONSET_DIR/icon_128x128.png" >/dev/null
    sips -z 256 256 "$ICON_SRC" --out "$ICONSET_DIR/icon_128x128@2x.png" >/dev/null
    sips -z 256 256 "$ICON_SRC" --out "$ICONSET_DIR/icon_256x256.png" >/dev/null
    sips -z 512 512 "$ICON_SRC" --out "$ICONSET_DIR/icon_256x256@2x.png" >/dev/null
    sips -z 512 512 "$ICON_SRC" --out "$ICONSET_DIR/icon_512x512.png" >/dev/null
    cp "$ICON_SRC" "$ICONSET_DIR/icon_512x512@2x.png"

    iconutil -c icns "$ICONSET_DIR" -o "$ICON_ICNS"
fi

cat > "$CONTENTS_DIR/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleDisplayName</key>
    <string>Stellar Wing</string>
    <key>CFBundleExecutable</key>
    <string>stellar-wing</string>
    <key>CFBundleIdentifier</key>
    <string>local.gaotao.stellar-wing</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleName</key>
    <string>Stellar Wing</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${APP_VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${APP_VERSION}</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.games</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
PLIST

cp "$BUILD_DIR/$BIN_NAME" "$MACOS_DIR/$BIN_NAME"
chmod +x "$MACOS_DIR/$BIN_NAME"

if [[ -n "${MACOS_CODESIGN_IDENTITY:-}" ]]; then
    codesign --force --deep --options runtime --sign "$MACOS_CODESIGN_IDENTITY" "$APP_DIR"
fi

if [[ -n "${MACOS_NOTARY_APPLE_ID:-}" && -n "${MACOS_NOTARY_TEAM_ID:-}" && -n "${MACOS_NOTARY_PASSWORD:-}" ]]; then
    ZIP_PATH="$DIST_DIR/$APP_NAME.notary.zip"
    rm -f "$ZIP_PATH"
    ditto -c -k --sequesterRsrc --keepParent "$APP_DIR" "$ZIP_PATH"
    xcrun notarytool submit "$ZIP_PATH" \
        --apple-id "$MACOS_NOTARY_APPLE_ID" \
        --team-id "$MACOS_NOTARY_TEAM_ID" \
        --password "$MACOS_NOTARY_PASSWORD" \
        --wait
    xcrun stapler staple "$APP_DIR"
    rm -f "$ZIP_PATH"
fi

echo "Built app bundle at: $APP_DIR"
