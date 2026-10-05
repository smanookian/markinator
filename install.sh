#!/bin/sh
# Installs markinator for the current user. No sudo.
#
#   curl -fsSL https://raw.githubusercontent.com/smanookian/markinator/main/install.sh | sh
#
# What it does:
#   1. downloads the release binary to ~/.local/bin/markinator
#   2. adds the app entry to ~/.local/share/applications (shows up in the launcher)
#   3. adds the icon to ~/.local/share/icons
#
# Gets the newest release. For a set version:
#   curl -fsSL .../install.sh | MARKINATOR_VERSION=v0.1.6 sh
#
# Remove with:
#   rm ~/.local/bin/markinator ~/.local/share/applications/markinator.desktop \
#      ~/.local/share/icons/hicolor/scalable/apps/markinator.svg

set -eu

VERSION="${MARKINATOR_VERSION:-latest}"
REPO="https://github.com/smanookian/markinator"
RAW="https://raw.githubusercontent.com/smanookian/markinator/main"

BIN="$HOME/.local/bin/markinator"
DESKTOP="$HOME/.local/share/applications/markinator.desktop"
ICON="$HOME/.local/share/icons/hicolor/scalable/apps/markinator.svg"

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) ;;
  *) echo "markinator: prebuilt binary is Linux x86_64 only. Build from source: $REPO" >&2; exit 1 ;;
esac

command -v curl >/dev/null || { echo "markinator: curl is required" >&2; exit 1; }

if [ "$VERSION" = latest ]; then
  URL="$REPO/releases/latest/download/markinator"
else
  URL="$REPO/releases/download/$VERSION/markinator"
fi

echo "Installing markinator ($VERSION)"
curl -fsSL --create-dirs -o "$BIN" "$URL"
chmod +x "$BIN"
curl -fsSL --create-dirs -o "$DESKTOP" "$RAW/markinator.desktop"
curl -fsSL --create-dirs -o "$ICON" "$RAW/icons/markinator.svg"

echo "  binary   $BIN"
echo "  launcher $DESKTOP"
echo "  icon     $ICON"

case ":$PATH:" in
  *":$HOME/.local/bin:"*) ;;
  *) echo "note: ~/.local/bin is not on your PATH; the launcher will still work." ;;
esac

echo "Done. Run: markinator notes.md"
