#!/usr/bin/env bash
# prepare-deb.sh — Bridge between versioned Tauri .deb output and the
# predictable drop-app.deb path expected by the Flatpak manifest.
#
# Usage:
#   ./flatpak/prepare-deb.sh
#
# Run this after `pnpm tauri build --bundles deb` to create a symlink
# at flatpak/drop-app.deb pointing to the actual versioned .deb file
# in the Tauri build output directory.
#
# CI note: In GitHub Actions, the Tauri build step produces the .deb
# into src-tauri/target/release/bundle/deb/. This script is called before
# flatpak-builder so the manifest's predictable path resolves correctly.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEB_DIR="${SCRIPT_DIR}/../src-tauri/target/release/bundle/deb"

if [ ! -d "$DEB_DIR" ]; then
  echo "Error: Tauri build output directory not found: $DEB_DIR" >&2
  echo "Make sure to run 'pnpm tauri build --bundles deb' first." >&2
  exit 1
fi

# Find the actual .deb — use glob to handle version changes
shopt -s nullglob
debs=("$DEB_DIR"/*_amd64.deb)
shopt -u nullglob

if [ ${#debs[@]} -eq 0 ]; then
  echo "Error: No .deb file found in $DEB_DIR" >&2
  echo "Expected a file matching *_amd64.deb" >&2
  exit 1
fi

# Use the first match (there should only be one)
actual_deb="${debs[0]}"
link_target="$SCRIPT_DIR/drop-app.deb"

ln -sf "$(realpath "$actual_deb")" "$link_target"
echo "Created symlink: $link_target -> $(realpath "$actual_deb")"