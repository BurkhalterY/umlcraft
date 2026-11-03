#!/usr/bin/env bash
# Write VERSION (or the first argument) into Cargo.toml's package version, for a release build only:
# the binary then reports it (About box, CARGO_PKG_VERSION). The change is not meant to be committed.
set -euo pipefail
cd "$(dirname "$0")/.."
v="${1:-${VERSION:?set VERSION or pass the version}}"
awk -v v="$v" '!done && /^version = "/ { print "version = \"" v "\""; done = 1; next } { print }' Cargo.toml >Cargo.toml.tmp
mv Cargo.toml.tmp Cargo.toml
grep -m1 '^version = ' Cargo.toml
