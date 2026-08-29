#!/usr/bin/env bash
# Install MyNote system-wide (Linux/GNOME).
# Builds a release binary and installs it along with the desktop entry,
# icon, and appstream metadata.

set -euo pipefail

PREFIX="${PREFIX:-/usr/local}"
BINDIR="${PREFIX}/bin"
DATADIR="${PREFIX}/share"
APP_ID="org.mynote.MyNote"
APP_ID_DIR="org.mynote.MyNote"
ICON_SRC="data/icons/hicolor"

echo "==> Building release binary..."
cargo build --release

echo "==> Installing to ${PREFIX}..."
install -Dm755 "target/release/mynote" "${BINDIR}/mynote"

install -Dm644 "data/${APP_ID}.desktop" "${DATADIR}/applications/${APP_ID}.desktop"
install -Dm644 "data/${APP_ID}.metainfo.xml" "${DATADIR}/metainfo/${APP_ID}.metainfo.xml"

# Install the SVG app icon into the hicolor theme.
install -Dm644 "${ICON_SRC}/scalable/apps/${APP_ID}.svg" \
    "${DATADIR}/icons/hicolor/scalable/apps/${APP_ID}.svg"

echo "==> Refreshing icon cache (if available)..."

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${DATADIR}/icons/hicolor" >/dev/null 2>&1 || true
fi

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${DATADIR}/applications" >/dev/null 2>&1 || true
fi

echo
echo "MyNote installed. Launch it with:  mynote"
echo
echo "To uninstall, remove:"
echo "  ${BINDIR}/mynote"
echo "  ${DATADIR}/applications/${APP_ID}.desktop"
echo "  ${DATADIR}/metainfo/${APP_ID}.metainfo.xml"
echo "  ${DATADIR}/icons/hicolor/scalable/apps/${APP_ID}.svg"
