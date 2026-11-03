#!/usr/bin/env bash
# Imports the Developer ID certificate (APPLE_CERTIFICATE: base64 .p12, APPLE_CERTIFICATE_PASSWORD)
# into a temporary keychain and prints the signing identity for later steps.
set -euo pipefail
keychain="$RUNNER_TEMP/umlcraft-signing.keychain-db"
echo "$APPLE_CERTIFICATE" | base64 --decode > "$RUNNER_TEMP/cert.p12"
security create-keychain -p "$KEYCHAIN_PASSWORD" "$keychain"
security set-keychain-settings -lut 21600 "$keychain"
security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$keychain"
security import "$RUNNER_TEMP/cert.p12" -P "$APPLE_CERTIFICATE_PASSWORD" -A -t cert -f pkcs12 -k "$keychain"
security set-key-partition-list -S apple-tool:,apple: -k "$KEYCHAIN_PASSWORD" "$keychain" >/dev/null
security list-keychains -d user -s "$keychain" $(security list-keychains -d user | tr -d '"')
rm "$RUNNER_TEMP/cert.p12"
identity="$(security find-identity -v -p codesigning "$keychain" | sed -n 's/.*"\(Developer ID Application[^"]*\)".*/\1/p' | head -n1)"
echo "MACOS_SIGNING_IDENTITY=$identity" >> "$GITHUB_ENV"
