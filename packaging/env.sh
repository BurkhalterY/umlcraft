# Shared settings for the packaging scripts (sourced, not run).
APP_ID="io.github.burkhaltery.UMLCraft"
APP_NAME="UMLCraft"
# version: $VERSION if set, else the one in Cargo.toml
if [ -z "${VERSION:-}" ]; then
  VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)"
fi
