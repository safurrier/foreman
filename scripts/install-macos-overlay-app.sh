#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="Foreman.app"
OLD_APP_NAME="Foreman Overlay.app"
SRC="$ROOT/apps/macos-overlay/dist/$APP_NAME"
OLD_SRC="$ROOT/apps/macos-overlay/dist/$OLD_APP_NAME"
DEST_DIR="${FOREMAN_OVERLAY_INSTALL_DIR:-$HOME/Applications}"
DEST="$DEST_DIR/$APP_NAME"
OLD_DEST="$DEST_DIR/$OLD_APP_NAME"
SYSTEM_DEST="/Applications/$APP_NAME"
SYSTEM_OLD_DEST="/Applications/$OLD_APP_NAME"
LSREGISTER="/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"

"$ROOT/scripts/build-macos-overlay-app.sh"
mkdir -p "$DEST_DIR"

# Stop every known Foreman overlay app process before replacing bundles. Raycast
# and Spotlight can otherwise keep reusing an already-running stale app.
pkill -f '/Foreman Overlay\.app/Contents/MacOS/foreman-overlay' >/dev/null 2>&1 || true
pkill -f '/Foreman\.app/Contents/MacOS/foreman-overlay' >/dev/null 2>&1 || true

# Unregister every historical/build/install location before copying the fresh
# bundle. This prevents LaunchServices/Raycast/Spotlight from preferring the
# dist/ build artifact or the old prototype bundle.
for app in "$SRC" "$OLD_SRC" "$DEST" "$OLD_DEST" "$SYSTEM_DEST" "$SYSTEM_OLD_DEST"; do
  "$LSREGISTER" -u "$app" >/dev/null 2>&1 || true
done

# Remove stale prototype bundles. Leave /Applications alone unless this task is
# explicitly installing there; deleting system-wide apps unexpectedly would be
# too surprising.
rm -rf "$DEST" "$OLD_DEST" "$OLD_SRC"
if [ "$DEST" = "$SYSTEM_DEST" ]; then
  rm -rf "$SYSTEM_OLD_DEST"
fi

cp -R "$SRC" "$DEST"

# Make the copied bundle look freshly changed so icon/app caches notice icon and
# metadata updates even when CFBundleIdentifier is unchanged.
touch "$DEST" "$DEST/Contents/Info.plist"
if [ -f "$DEST/Contents/Resources/ForemanIcon.icns" ]; then
  touch "$DEST/Contents/Resources/ForemanIcon.icns"
fi

# Register only the installed app. Then unregister and remove the dist build
# artifact because LaunchServices/Raycast can rediscover any .app left under the
# repo after validation/build tasks.
"$LSREGISTER" -f "$DEST" >/dev/null 2>&1 || true
"$LSREGISTER" -u "$SRC" >/dev/null 2>&1 || true
rm -rf "$SRC"
mdimport "$DEST" >/dev/null 2>&1 || true
qlmanage -r >/dev/null 2>&1 || true
qlmanage -r cache >/dev/null 2>&1 || true

# Show what launchers should see now.
echo "Installed $DEST"
echo "Launch with: open -a Foreman"
echo "If Raycast/Spotlight still show a stale icon or app, quit/reopen Raycast or wait for indexing."
