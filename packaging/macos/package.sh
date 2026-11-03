#!/usr/bin/env bash
# Builds a universal (Apple Silicon + Intel) UMLCraft.app and writes to dist/release:
#   UMLCraft-<version>-macos-universal.dmg and .zip
# Signing: with MACOS_SIGNING_IDENTITY set (a "Developer ID Application" identity in the keychain)
# the app is signed with the hardened runtime, and with APPLE_ID, APPLE_PASSWORD and APPLE_TEAM_ID
# also notarized; otherwise it is signed ad hoc so it still starts on Apple Silicon.
set -euo pipefail
cd "$(dirname "$0")/../.."
source packaging/env.sh

targets=(aarch64-apple-darwin x86_64-apple-darwin)
for t in "${targets[@]}"; do
  rustup target add "$t" >/dev/null
  cargo build --release --target "$t"
done

app="dist/macos/$APP_NAME.app"
rm -rf dist/macos
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources" dist/release
lipo -create -output "$app/Contents/MacOS/umlcraft" \
  target/aarch64-apple-darwin/release/umlcraft target/x86_64-apple-darwin/release/umlcraft
sed -e "s/@VERSION@/$VERSION/g" -e "s/@APP_ID@/$APP_ID/g" packaging/macos/Info.plist.in > "$app/Contents/Info.plist"
cp packaging/icons/umlcraft.icns "$app/Contents/Resources/umlcraft.icns"
plutil -lint "$app/Contents/Info.plist"

if [ -n "${MACOS_SIGNING_IDENTITY:-}" ]; then
  codesign --force --options runtime --timestamp --entitlements packaging/macos/entitlements.plist \
    --sign "$MACOS_SIGNING_IDENTITY" "$app"
else
  echo "no MACOS_SIGNING_IDENTITY: signing ad hoc"
  codesign --force --sign - "$app"
fi
codesign --verify --deep --strict "$app"

base="dist/release/$APP_NAME-$VERSION-macos-universal"
# disk image with a link to /Applications
rm -rf dist/dmg && mkdir -p dist/dmg
cp -R "$app" dist/dmg/
ln -s /Applications dist/dmg/Applications
hdiutil create -volname "$APP_NAME" -srcfolder dist/dmg -ov -format UDZO "$base.dmg"
ditto -c -k --keepParent "$app" "$base.zip"

if [ -n "${MACOS_SIGNING_IDENTITY:-}" ] && [ -n "${APPLE_ID:-}" ] && [ -n "${APPLE_PASSWORD:-}" ] && [ -n "${APPLE_TEAM_ID:-}" ]; then
  codesign --force --timestamp --sign "$MACOS_SIGNING_IDENTITY" "$base.dmg"
  xcrun notarytool submit "$base.dmg" --apple-id "$APPLE_ID" --password "$APPLE_PASSWORD" --team-id "$APPLE_TEAM_ID" --wait
  xcrun stapler staple "$base.dmg"
else
  echo "notarization skipped (needs MACOS_SIGNING_IDENTITY, APPLE_ID, APPLE_PASSWORD, APPLE_TEAM_ID)"
fi
ls -la dist/release
