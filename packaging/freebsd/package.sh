#!/usr/bin/env bash
# Builds UMLCraft on FreeBSD and writes to dist/release:
#   umlcraft-<version>.pkg                       install with: pkg add umlcraft-<version>.pkg
#   umlcraft-<version>-freebsd-<arch>.tar.gz     portable archive
# Needs: rust, pkgconf, gtk3, libxkbcommon, mesa-libs (pkg install -y rust pkgconf gtk3 libxkbcommon mesa-libs)
set -euo pipefail
cd "$(dirname "$0")/../.."
source packaging/env.sh
ARCH="$(uname -m)"

cargo build --release

prefix=/usr/local
stage="dist/freebsd/stage"
rm -rf dist/freebsd
mkdir -p dist/release
install -d "$stage$prefix/bin" "$stage$prefix/share/applications" "$stage$prefix/share/mime/packages" "$stage$prefix/share/doc/umlcraft"
install -m 755 target/release/umlcraft "$stage$prefix/bin/umlcraft"
install -m 644 "packaging/linux/$APP_ID.desktop" "$stage$prefix/share/applications/$APP_ID.desktop"
install -m 644 "packaging/linux/$APP_ID.mime.xml" "$stage$prefix/share/mime/packages/$APP_ID.xml"
install -m 644 README.md "$stage$prefix/share/doc/umlcraft/README.md"
for n in 16 24 32 48 64 128 256 512; do
  install -d "$stage$prefix/share/icons/hicolor/${n}x${n}/apps"
  install -m 644 "packaging/icons/umlcraft-$n.png" "$stage$prefix/share/icons/hicolor/${n}x${n}/apps/$APP_ID.png"
done

# portable archive
name="umlcraft-$VERSION-freebsd-$ARCH"
mkdir -p "dist/freebsd/$name"
cp -R "$stage$prefix/" "dist/freebsd/$name/"
tar -C dist/freebsd -czf "dist/release/$name.tar.gz" "$name"

# native package
deps=""
for d in gtk3 libxkbcommon mesa-libs fontconfig; do
  if pkg info -e "$d"; then
    deps="$deps \"$d\": {origin: \"$(pkg query '%o' "$d")\", version: \"$(pkg query '%v' "$d")\"},"
  fi
done
cat > dist/freebsd/manifest <<MANIFEST
name: umlcraft
version: "$VERSION"
origin: devel/umlcraft
comment: "UML modeler for class and use case diagrams"
desc: "UMLCraft opens and saves .mdj model files, keeping everything it does not edit exactly as it was."
maintainer: "yannis@burkhalter.dev"
www: "https://github.com/BurkhalterY/umlcraft"
prefix: "$prefix"
licenselogic: single
licenses: [MIT]
deps: {${deps%,}}
MANIFEST
(cd "$stage" && find ".$prefix" -type f | sed "s|^\.||") > dist/freebsd/plist
pkg create -M dist/freebsd/manifest -r "$stage" -p dist/freebsd/plist -o dist/release
ls -la dist/release
