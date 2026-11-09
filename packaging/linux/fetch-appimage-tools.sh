#!/usr/bin/env bash
# Downloads appimagetool and the AppImage type 2 runtime for one architecture:
#   <dir>/appimagetool.AppImage   the tool
#   <dir>/runtime                 the runtime, passed to appimagetool with --runtime-file
# Usage: packaging/linux/fetch-appimage-tools.sh <x86_64|aarch64> <dir>
#
# GitHub release downloads sometimes answer HTTP 504 for hours. Each file is retried, and if it
# still fails, a copy already in <dir> (restored from the Actions cache) is used instead.
# Without the runtime file, appimagetool downloads the runtime itself, with no retry.
set -euo pipefail
arch="$1"
dir="$2"
mkdir -p "$dir"

fetch() { # <url> <file name in dir>
  if curl -fsSL --retry 6 --retry-delay 10 --retry-all-errors --connect-timeout 30 \
    -o "$dir/$2.part" "$1"; then
    mv "$dir/$2.part" "$dir/$2"
    echo "downloaded $2"
  else
    rm -f "$dir/$2.part"
    if [ -s "$dir/$2" ]; then
      echo "::warning::Could not download $1; using the cached $2."
    else
      echo "::error::Could not download $1, and no cached $2 exists."
      return 1
    fi
  fi
}

fetch "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-$arch.AppImage" appimagetool.AppImage
fetch "https://github.com/AppImage/type2-runtime/releases/download/continuous/runtime-$arch" runtime
chmod +x "$dir/appimagetool.AppImage"
