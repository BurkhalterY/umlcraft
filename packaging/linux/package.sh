#!/usr/bin/env bash
# Builds UMLCraft for Linux and writes to dist/release:
#   umlcraft-<version>-linux-<arch>.tar.gz   always
#   umlcraft_<version>_<debarch>.deb, umlcraft-<version>.<rpmarch>.rpm   when nfpm is installed
#   UMLCraft-<version>-<arch>.AppImage       when appimagetool is installed
# Environment: VERSION (default: Cargo.toml), SKIP_BUILD=1 to reuse target/release/umlcraft.
set -euo pipefail
cd "$(dirname "$0")/../.."
source packaging/env.sh

ARCH="$(uname -m)"
case "$ARCH" in
  x86_64) NFPM_ARCH=amd64 ;;
  aarch64 | arm64) ARCH=aarch64; NFPM_ARCH=arm64 ;;
  *) NFPM_ARCH="$ARCH" ;;
esac
export VERSION NFPM_ARCH

if [ "${SKIP_BUILD:-0}" != 1 ]; then
  cargo build --release
fi

rm -rf dist/staging dist/AppDir
mkdir -p dist/release
stage=dist/staging/usr
install -Dm755 target/release/umlcraft "$stage/bin/umlcraft"
install -Dm644 "packaging/linux/$APP_ID.desktop" "$stage/share/applications/$APP_ID.desktop"
install -Dm644 "packaging/linux/$APP_ID.mime.xml" "$stage/share/mime/packages/$APP_ID.xml"
sed -e "s/@VERSION@/$VERSION/" -e "s/@DATE@/$(date -u +%Y-%m-%d)/" "packaging/linux/$APP_ID.metainfo.xml.in" > "$stage/share/metainfo-$APP_ID.tmp"
install -Dm644 "$stage/share/metainfo-$APP_ID.tmp" "$stage/share/metainfo/$APP_ID.metainfo.xml"
rm "$stage/share/metainfo-$APP_ID.tmp"
for n in 16 24 32 48 64 128 256 512; do
  install -Dm644 "packaging/icons/umlcraft-$n.png" "$stage/share/icons/hicolor/${n}x${n}/apps/$APP_ID.png"
done
install -Dm644 README.md "$stage/share/doc/umlcraft/README.md"

# portable archive
name="umlcraft-$VERSION-linux-$ARCH"
rm -rf "dist/$name"
mkdir -p "dist/$name"
cp -a "$stage/." "dist/$name/"
tar -C dist -czf "dist/release/$name.tar.gz" "$name"
rm -rf "dist/$name"
echo "wrote dist/release/$name.tar.gz"

# deb and rpm
if command -v nfpm >/dev/null 2>&1; then
  nfpm package --config packaging/linux/nfpm.yaml --packager deb --target dist/release/
  nfpm package --config packaging/linux/nfpm.yaml --packager rpm --target dist/release/
else
  echo "nfpm not found: skipping .deb and .rpm"
fi

# AppImage
if command -v appimagetool >/dev/null 2>&1; then
  appdir=dist/AppDir
  mkdir -p "$appdir"
  cp -a "$stage" "$appdir/usr"
  cp packaging/linux/AppRun "$appdir/AppRun"
  cp "packaging/linux/$APP_ID.desktop" "$appdir/$APP_ID.desktop"
  cp packaging/icons/umlcraft-256.png "$appdir/$APP_ID.png"
  ARCH="$ARCH" appimagetool --no-appstream "$appdir" "dist/release/UMLCraft-$VERSION-$ARCH.AppImage"
else
  echo "appimagetool not found: skipping the AppImage"
fi
ls -la dist/release
