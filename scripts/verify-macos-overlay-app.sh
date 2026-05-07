#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP="$ROOT/apps/macos-overlay/dist/Foreman.app"
PLIST="$APP/Contents/Info.plist"
EXE="$APP/Contents/MacOS/foreman-overlay"
FOREMAN="$APP/Contents/Resources/foreman"
ICON="$APP/Contents/Resources/ForemanIcon.icns"
VALIDATION_DIR="$ROOT/.ai/validation/macos-overlay/app-bundle"
LSREGISTER="/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"
mkdir -p "$VALIDATION_DIR"

cleanup() {
  # Running the bundle executable from dist/ can make LaunchServices discover the
  # build artifact. Keep validation from poisoning Spotlight/Raycast/open -a;
  # only the install task should register Foreman.app.
  "$LSREGISTER" -u "$APP" >/dev/null 2>&1 || true
}
trap cleanup EXIT

"$ROOT/scripts/build-macos-overlay-app.sh" >/dev/null

[ -d "$APP" ] || { echo "missing app bundle: $APP" >&2; exit 1; }
[ -x "$EXE" ] || { echo "missing executable: $EXE" >&2; exit 1; }
[ -x "$FOREMAN" ] || { echo "missing bundled foreman binary: $FOREMAN" >&2; exit 1; }
[ -f "$ICON" ] || { echo "missing app icon: $ICON" >&2; exit 1; }
[ -f "$PLIST" ] || { echo "missing Info.plist: $PLIST" >&2; exit 1; }

bundle_id="$(/usr/libexec/PlistBuddy -c 'Print CFBundleIdentifier' "$PLIST")"
executable="$(/usr/libexec/PlistBuddy -c 'Print CFBundleExecutable' "$PLIST")"
icon_file="$(/usr/libexec/PlistBuddy -c 'Print CFBundleIconFile' "$PLIST")"
package_type="$(/usr/libexec/PlistBuddy -c 'Print CFBundlePackageType' "$PLIST")"
[ "$bundle_id" = "dev.foreman.app" ] || { echo "unexpected bundle id: $bundle_id" >&2; exit 1; }
[ "$executable" = "foreman-overlay" ] || { echo "unexpected executable: $executable" >&2; exit 1; }
[ "$icon_file" = "ForemanIcon" ] || { echo "unexpected icon file: $icon_file" >&2; exit 1; }
[ "$package_type" = "APPL" ] || { echo "unexpected package type: $package_type" >&2; exit 1; }

LOG="$VALIDATION_DIR/launch.log"
READY_FILE="$VALIDATION_DIR/ready.txt"
rm -f "$LOG" "$READY_FILE"
FOREMAN_OVERLAY_ACTIVATION_POLICY=accessory \
FOREMAN_OVERLAY_SHOW_ON_LAUNCH=0 \
FOREMAN_OVERLAY_READY_FILE="$READY_FILE" \
FOREMAN_OVERLAY_EXIT_AFTER_READY=1 \
"$EXE" >"$LOG" 2>&1 &
APP_PID=$!

for _ in $(seq 1 40); do
  if [ -s "$READY_FILE" ]; then
    break
  fi
  if ! kill -0 "$APP_PID" 2>/dev/null; then
    echo "bundled app exited early; see $LOG" >&2
    exit 1
  fi
  sleep 0.1
done

wait "$APP_PID" >/dev/null 2>&1 || true
[ -s "$READY_FILE" ] || { echo "bundled app did not write ready file" >&2; exit 1; }

app_rel="${APP#$ROOT/}"
exe_rel="${EXE#$ROOT/}"
foreman_rel="${FOREMAN#$ROOT/}"
icon_rel="${ICON#$ROOT/}"
ready_rel="${READY_FILE#$ROOT/}"
cat > "$VALIDATION_DIR/summary.md" <<EOF
# macOS Overlay App Bundle Smoke

- App: $app_rel
- Bundle ID: $bundle_id
- Executable: $exe_rel
- Bundled Foreman: $foreman_rel
- Icon: $icon_rel
- Ready file: $ready_rel
- Verified: $(date -u +%Y-%m-%dT%H:%M:%SZ)
EOF

echo "macOS overlay app bundle smoke passed"
echo "Summary: $VALIDATION_DIR/summary.md"
