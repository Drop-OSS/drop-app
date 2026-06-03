#!/usr/bin/env bash
# prepare-libs.sh — Copy host system tray libraries into flatpak/libs/
# for bundling in the Flatpak build.
#
# These libraries (libayatana-appindicator3, libdbusmenu) are not available
# in the GNOME Platform runtime, so we bundle them from the host.
#
# Usage:
#   ./flatpak/prepare-libs.sh
#
# In CI (Ubuntu), run this after:
#   sudo apt-get install -y libayatana-appindicator3-dev libdbusmenu-gtk3-dev

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIBS_DIR="${SCRIPT_DIR}/libs"
mkdir -p "$LIBS_DIR"

# Library files needed at runtime — copied from host system
declare -A REQUIRED_LIBS=(
  ["libayatana-appindicator3.so.1"]="libayatana-appindicator3.so.1"
  ["libayatana-appindicator3.so.1.0.0"]="libayatana-appindicator3.so.1.0.0"
  ["libayatana-ido3-0.4.so.0"]="libayatana-ido3-0.4.so.0"
  ["libayatana-ido3-0.4.so.0.0.0"]="libayatana-ido3-0.4.so.0.0.0"
  ["libayatana-indicator3.so.7"]="libayatana-indicator3.so.7"
  ["libayatana-indicator3.so.7.0.0"]="libayatana-indicator3.so.7.0.0"
  ["libdbusmenu-glib.so.4"]="libdbusmenu-glib.so.4"
  ["libdbusmenu-glib.so.4.0.12"]="libdbusmenu-glib.so.4.0.12"
  ["libdbusmenu-gtk3.so.4"]="libdbusmenu-gtk3.so.4"
  ["libdbusmenu-gtk3.so.4.0.12"]="libdbusmenu-gtk3.so.4.0.12"
)

all_found=true
for soname in "${!REQUIRED_LIBS[@]}"; do
  libfile="${REQUIRED_LIBS[$soname]}"
  # Search common library paths
  found=$(find /usr/lib64 /usr/lib /lib64 /lib -name "$libfile" -type f 2>/dev/null | head -1)
  if [ -n "$found" ]; then
    cp -f "$found" "$LIBS_DIR/$soname"
    echo "  ✓ $soname ($(du -h "$found" | cut -f1))"
  else
    # Try with apt-file or ldconfig
    found_ld=$(ldconfig -p 2>/dev/null | grep "$libfile" | awk '{print $NF}' | head -1)
    if [ -n "$found_ld" ] && [ -f "$found_ld" ]; then
      cp -f "$found_ld" "$LIBS_DIR/$soname"
      echo "  ✓ $soname via ldconfig ($(du -h "$found_ld" | cut -f1))"
    else
      echo "  ✗ $soname NOT FOUND — install libayatana-appindicator3-dev and libdbusmenu-gtk3-dev" >&2
      all_found=false
    fi
  fi
done

if [ "$all_found" = false ]; then
  echo "Error: Some required libraries were not found." >&2
  echo "Install them and re-run:" >&2
  echo "  # Fedora: sudo dnf install libayatana-appindicator-gtk3-devel libdbusmenu-gtk3-devel" >&2
  echo "  # Ubuntu: sudo apt-get install libayatana-appindicator3-dev libdbusmenu-gtk3-dev" >&2
  exit 1
fi

echo "All system tray libraries prepared in $LIBS_DIR"