#!/usr/bin/env bash
# build.sh — Build the Drop Desktop Flatpak from scratch.
#
# Usage:
#   ./flatpak/build.sh            # Build flatpak
#   ./flatpak/build.sh --bundle   # Build and create portable .flatpak bundle
#   ./flatpak/build.sh --clean    # Clean build artifacts only
#
# Prerequisites:
#   - flatpak, flatpak-builder installed
#   - GNOME Platform/SDK 48 installed:
#       flatpak install --user flathub org.gnome.Platform//48 org.gnome.Sdk//48
#   - pnpm, node, rust/cargo installed for Tauri build
#
# The build process:
#   1. Build the Tauri app → produces .deb
#   2. Prepare system tray libraries from host
#   3. Create symlink from versioned .deb to flatpak/drop-app.deb
#   4. Run flatpak-builder
#   5. Optionally bundle to .flatpak file

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BUNDLE_FILE="${PROJECT_DIR}/org.droposs.client.flatpak"
BUILD_DIR="${PROJECT_DIR}/flatpak-build-dir"

# ── Parse arguments ──
do_bundle=false
do_clean=false
for arg in "$@"; do
  case "$arg" in
    --bundle) do_bundle=true ;;
    --clean)  do_clean=true ;;
    --help)
      echo "Usage: $0 [--bundle] [--clean]"
      echo "  --bundle   Create portable .flatpak bundle after build"
      echo "  --clean    Remove build artifacts only"
      exit 0
      ;;
    *)
      echo "Unknown option: $arg"
      echo "Usage: $0 [--bundle] [--clean]"
      exit 1
      ;;
  esac
done

# ── Clean mode ──
if [ "$do_clean" = true ]; then
  echo "==> Cleaning build artifacts..."
  rm -rf "$BUILD_DIR"
  rm -f "$SCRIPT_DIR/drop-app.deb"
  rm -rf "$SCRIPT_DIR/libs"
  rm -f "$BUNDLE_FILE"
  rm -rf "$PROJECT_DIR/.flatpak-builder"
  echo "    Done."
  exit 0
fi

# ── Step 0: Verify prerequisites ──
echo "==> [1/4] Verifying prerequisites..."

if ! command -v flatpak &>/dev/null; then
  echo "Error: flatpak not found. Install it first." >&2
  exit 1
fi
if ! command -v flatpak-builder &>/dev/null; then
  echo "Error: flatpak-builder not found. Install it first." >&2
  exit 1
fi

# Check GNOME runtime/SDK are available
if ! flatpak info org.gnome.Platform//48 &>/dev/null; then
  echo "Installing GNOME Platform 48..."
  flatpak install --user flathub org.gnome.Platform//48 org.gnome.Sdk//48 -y
fi

echo "    All prerequisites met."

# ── Step 1: Build Tauri app ──
echo "==> [2/4] Building Tauri app..."
cd "$PROJECT_DIR"

# Build frontend + Tauri binary + .deb package
pnpm tauri build --bundles deb
echo "    Tauri build complete."

# ── Step 2: Prepare system tray libraries ──
echo "==> [3/4] Preparing system tray libraries..."
bash "$SCRIPT_DIR/prepare-libs.sh"

# ── Step 3: Prepare .deb for flatpak ──
echo "==> [4/4] Building Flatpak..."
bash "$SCRIPT_DIR/prepare-deb.sh"

# ── Step 4: Run flatpak-builder ──
flatpak-builder --user --force-clean "$BUILD_DIR" "$SCRIPT_DIR/org.droposs.client.yml"
flatpak-builder --user --install "$BUILD_DIR" "$SCRIPT_DIR/org.droposs.client.yml"

echo ""
echo "=== Flatpak build complete ==="
echo "  Run: flatpak run org.droposs.client"

# ── Optional: create portable bundle ──
if [ "$do_bundle" = true ]; then
  echo "==> Creating portable bundle..."
  flatpak build-bundle ~/.local/share/flatpak/repo "$BUNDLE_FILE" org.droposs.client
  echo "  Bundle: $BUNDLE_FILE ($(du -h "$BUNDLE_FILE" | cut -f1))"
fi

echo ""