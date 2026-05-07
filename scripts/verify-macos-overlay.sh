#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VALIDATION_DIR="$ROOT/.ai/validation/macos-overlay"
mkdir -p "$VALIDATION_DIR"
SUMMARY="$VALIDATION_DIR/verify-summary.md"

run_step() {
  local name="$1"
  shift
  echo "==> $name"
  "$@"
}

run_step "Rust control API unit tests" cargo test --manifest-path "$ROOT/Cargo.toml" control_api --lib
run_step "Swift overlay unit tests" swift test --package-path "$ROOT/apps/macos-overlay"
run_step "macOS overlay UI event XCTest gauntlet" env FOREMAN_OVERLAY_RUN_UI_TESTS=1 FOREMAN_OVERLAY_SKIP_BUILD=1 swift test --package-path "$ROOT/apps/macos-overlay" --filter ForemanOverlayUITests
run_step "Control API real tmux smoke" "$ROOT/scripts/smoke-control-api-tmux.sh"
run_step "macOS overlay headless snapshots" "$ROOT/scripts/render-macos-overlay-snapshots.sh"
run_step "macOS overlay app bundle smoke" "$ROOT/scripts/verify-macos-overlay-app.sh"

SNAPSHOT_BIN="$ROOT/apps/macos-overlay/.build/debug/foreman-overlay-snapshot"
while IFS= read -r state; do
  [ -n "$state" ] || continue
  if [ ! -s "$VALIDATION_DIR/headless-snapshots/$state.png" ]; then
    echo "missing headless snapshot artifact: $VALIDATION_DIR/headless-snapshots/$state.png" >&2
    exit 1
  fi
done < <("$SNAPSHOT_BIN" --list-states)

cat > "$SUMMARY" <<EOF
# macOS Overlay Verification

- Rust control API unit tests: passed
- Swift overlay unit tests: passed
- Fake-Foreman action gauntlet: passed
- Control API real tmux smoke: passed
- Headless SwiftUI snapshots: passed
- App bundle smoke: passed
- Snapshots: .ai/validation/macos-overlay/headless-snapshots/
- Fake Foreman calls: .ai/validation/macos-overlay/fake-foreman-calls.log
- Verified: $(date -u +%Y-%m-%dT%H:%M:%SZ)
EOF

echo "macOS overlay verification passed"
echo "Summary: $SUMMARY"
