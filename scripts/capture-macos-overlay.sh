#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${FOREMAN_OVERLAY_CAPTURE_DIR:-$ROOT/.ai/validation/macos-overlay}"
STATE="${FOREMAN_OVERLAY_CAPTURE_STATE:-attention}"
mkdir -p "$OUT_DIR"

state_fixture() {
  case "$1" in
    attention) echo "$ROOT/apps/macos-overlay/Fixtures/agents-attention.json" ;;
    empty) echo "$ROOT/apps/macos-overlay/Fixtures/agents-empty.json" ;;
    long-path) echo "$ROOT/apps/macos-overlay/Fixtures/agents-long-path.json" ;;
    long-preview) echo "$ROOT/apps/macos-overlay/Fixtures/agents-long-preview.json" ;;
    many) echo "$ROOT/apps/macos-overlay/Fixtures/agents-many.json" ;;
    mixed-status) echo "$ROOT/apps/macos-overlay/Fixtures/agents-mixed-status.json" ;;
    error) echo "__error__" ;;
    *) echo "unknown capture state: $1" >&2; exit 64 ;;
  esac
}

capture_one() {
  local state="$1"
  local fixture
  fixture="$(state_fixture "$state")"
  local state_dir="$OUT_DIR/screenshots"
  mkdir -p "$state_dir"

  local fake_dir
  fake_dir="$(mktemp -d)"
  local app_pid=""
  cleanup_one() {
    set +e
    if [ -n "$app_pid" ] && kill -0 "$app_pid" 2>/dev/null; then
      kill "$app_pid" >/dev/null 2>&1 || true
      wait "$app_pid" >/dev/null 2>&1 || true
    fi
    pkill -f 'foreman-overlay' >/dev/null 2>&1 || true
    rm -rf "$fake_dir"
  }
  trap cleanup_one RETURN

  local fake_foreman="$fake_dir/foreman"
  local call_log="$OUT_DIR/fake-foreman-${state}-calls.log"
  local stdin_log="$OUT_DIR/fake-foreman-${state}-stdin.log"
  local app_log="$OUT_DIR/foreman-overlay-${state}.log"
  local screenshot="$state_dir/${state}.png"
  local window_id_file="$fake_dir/window-id"
  local panel_snapshot="$fake_dir/panel.png"

  cat > "$fake_foreman" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "${FOREMAN_OVERLAY_CALL_LOG:?}"
case "${1:-}" in
  agents)
    if [ "${FOREMAN_OVERLAY_FAKE_ERROR:-0}" = "1" ]; then
      echo "fixture tmux unavailable: fake Foreman error" >&2
      exit 2
    fi
    cat "${FOREMAN_OVERLAY_FIXTURE_JSON:?}"
    ;;
  focus)
    pane=""
    while [ "$#" -gt 0 ]; do
      case "$1" in
        --pane) shift; pane="${1:-}" ;;
      esac
      shift || true
    done
    printf '{"schemaVersion":1,"ok":true,"action":"focus","paneId":"%s","bytesSent":null}\n' "$pane"
    ;;
  send)
    pane=""
    while [ "$#" -gt 0 ]; do
      case "$1" in
        --pane) shift; pane="${1:-}" ;;
      esac
      shift || true
    done
    bytes=$(cat | tee "${FOREMAN_OVERLAY_STDIN_LOG:?}" | wc -c | tr -d ' ')
    printf '{"schemaVersion":1,"ok":true,"action":"send","paneId":"%s","bytesSent":%s}\n' "$pane" "$bytes"
    ;;
  *)
    echo "fake foreman: unsupported command: $*" >&2
    exit 64
    ;;
esac
EOF
  chmod +x "$fake_foreman"

  : > "$call_log"
  : > "$stdin_log"
  : > "$app_log"

  local fake_error=0
  local launch_fixture="$fixture"
  if [ "$fixture" = "__error__" ]; then
    fake_error=1
    launch_fixture="$ROOT/apps/macos-overlay/Fixtures/agents-attention.json"
  fi

  pkill -f 'foreman-overlay' >/dev/null 2>&1 || true
  FOREMAN_OVERLAY_FOREMAN_PATH="$fake_foreman" \
  FOREMAN_OVERLAY_SHOW_ON_LAUNCH=1 \
  FOREMAN_OVERLAY_ACTIVATE="${FOREMAN_OVERLAY_ACTIVATE:-0}" \
  FOREMAN_OVERLAY_JOIN_ALL_SPACES="${FOREMAN_OVERLAY_JOIN_ALL_SPACES:-0}" \
  FOREMAN_OVERLAY_SCREEN_INDEX="${FOREMAN_OVERLAY_SCREEN_INDEX:-}" \
  FOREMAN_OVERLAY_WINDOW_ID_PATH="$window_id_file" \
  FOREMAN_OVERLAY_PANEL_SNAPSHOT_PATH="$panel_snapshot" \
  FOREMAN_OVERLAY_FIXTURE_JSON="$launch_fixture" \
  FOREMAN_OVERLAY_FAKE_ERROR="$fake_error" \
  FOREMAN_OVERLAY_CALL_LOG="$call_log" \
  FOREMAN_OVERLAY_STDIN_LOG="$stdin_log" \
  "$ROOT/apps/macos-overlay/.build/debug/foreman-overlay" >"$app_log" 2>&1 &
  app_pid=$!

  sleep "${FOREMAN_OVERLAY_CAPTURE_DELAY:-2}"
  if ! kill -0 "$app_pid" 2>/dev/null; then
    echo "foreman-overlay exited early for state $state; see $app_log" >&2
    exit 1
  fi

  for _ in $(seq 1 30); do
    if [ -s "$panel_snapshot" ]; then
      cp "$panel_snapshot" "$screenshot"
      break
    fi
    sleep 0.1
  done

  if [ ! -s "$screenshot" ]; then
    local window_id=""
    if [ -s "$window_id_file" ]; then
      window_id="$(tr -d '[:space:]' < "$window_id_file")"
    fi
    if [ -n "$window_id" ]; then
      screencapture -x -l "$window_id" "$screenshot" 2>/dev/null || screencapture -x "$screenshot"
    else
      screencapture -x "$screenshot"
    fi
  fi
  if [ ! -s "$screenshot" ]; then
    echo "missing screenshot for state $state: $screenshot" >&2
    exit 1
  fi
  grep -E '^agents --json( --pull-requests)?$' "$call_log" >/dev/null || {
    echo "fake foreman call log for $state did not include agents --json" >&2
    exit 1
  }

  echo "- $state: $screenshot"
  cleanup_one
  trap - RETURN
}

swift build --package-path "$ROOT/apps/macos-overlay" >/dev/null

SUMMARY="$OUT_DIR/summary.md"
{
  echo "# macOS Overlay Capture"
  echo
  echo "- Captured: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "- State request: $STATE"
  echo
  echo "## Screenshots"
} > "$SUMMARY"

if [ "$STATE" = "all" ]; then
  for state in attention empty error long-path long-preview many mixed-status; do
    capture_one "$state" | tee -a "$SUMMARY"
  done
else
  capture_one "$STATE" | tee -a "$SUMMARY"
  if [ "$STATE" = "attention" ]; then
    cp "$OUT_DIR/screenshots/attention.png" "$OUT_DIR/attention.png"
  fi
fi

# Preserve the original convenient artifact path when capturing all states.
if [ "$STATE" = "all" ] && [ -s "$OUT_DIR/screenshots/attention.png" ]; then
  cp "$OUT_DIR/screenshots/attention.png" "$OUT_DIR/attention.png"
fi

echo "Summary: $SUMMARY"
