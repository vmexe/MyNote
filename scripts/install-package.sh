#!/usr/bin/env bash
# Install MyNote from a prebuilt release package.
# Copy this script together with the `mynote` binary and `share/` folder,
# then run it. No build tools needed.
#
# By default it installs to /usr/local. Override with PREFIX=/usr, for example.

set -euo pipefail

PREFIX="${PREFIX:-/usr/local}"
BINDIR="${PREFIX}/bin"
DATADIR="${PREFIX}/share"
APP_ID="org.mynote.MyNote"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [ ! -x "${SCRIPT_DIR}/mynote" ]; then
    echo "Error: 'mynote' binary not found next to this script."
    echo "Put this script in the same folder as the 'mynote' binary and the 'share/' folder."
    exit 1
fi

echo "==> Installing MyNote to ${PREFIX}..."

install -Dm755 "${SCRIPT_DIR}/mynote" "${BINDIR}/mynote"
install -Dm644 "${SCRIPT_DIR}/share/applications/${APP_ID}.desktop" \
    "${DATADIR}/applications/${APP_ID}.desktop"
install -Dm644 "${SCRIPT_DIR}/share/metainfo/${APP_ID}.metainfo.xml" \
    "${DATADIR}/metainfo/${APP_ID}.metainfo.xml"
install -Dm644 "${SCRIPT_DIR}/share/icons/hicolor/scalable/apps/${APP_ID}.svg" \
    "${DATADIR}/icons/hicolor/scalable/apps/${APP_ID}.svg"

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${DATADIR}/icons/hicolor" >/dev/null 2>&1 || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${DATADIR}/applications" >/dev/null 2>&1 || true
fi

echo "MyNote installed. Launch it from your app menu or run:  mynote"
