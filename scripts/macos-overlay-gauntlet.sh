#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="$ROOT/.ai/validation/macos-overlay/gauntlet"
FIXTURE="$ROOT/apps/macos-overlay/Fixtures/agents-attention.json"
mkdir -p "$OUT_DIR"

FAKE_DIR="$(mktemp -d)"
cleanup() {
  set +e
  pkill -f 'foreman-overlay' >/dev/null 2>&1 || true
  rm -rf "$FAKE_DIR"
}
trap cleanup EXIT

FAKE_FOREMAN="$FAKE_DIR/foreman"
CALL_LOG="$OUT_DIR/fake-foreman-calls.log"
STDIN_LOG="$OUT_DIR/fake-foreman-stdin.log"
APP_LOG="$OUT_DIR/overlay.log"
SUMMARY="$OUT_DIR/summary.md"

cat > "$FAKE_FOREMAN" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "${FOREMAN_OVERLAY_CALL_LOG:?}"
case "${1:-}" in
  agents)
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
chmod +x "$FAKE_FOREMAN"

if [ "${FOREMAN_OVERLAY_SKIP_BUILD:-0}" != "1" ]; then
  swift build --package-path "$ROOT/apps/macos-overlay" >/dev/null
fi
: > "$CALL_LOG"
: > "$STDIN_LOG"
: > "$APP_LOG"

FOREMAN_OVERLAY_FOREMAN_PATH="$FAKE_FOREMAN" \
FOREMAN_OVERLAY_SHOW_ON_LAUNCH=1 \
FOREMAN_OVERLAY_SCRIPTED_GAUNTLET=1 \
FOREMAN_OVERLAY_TERMINAL_ACTIVATION=bundle:com.foreman.NonexistentTerminalFixture \
FOREMAN_OVERLAY_FIXTURE_JSON="$FIXTURE" \
FOREMAN_OVERLAY_CALL_LOG="$CALL_LOG" \
FOREMAN_OVERLAY_STDIN_LOG="$STDIN_LOG" \
"$ROOT/apps/macos-overlay/.build/debug/foreman-overlay" >"$APP_LOG" 2>&1 &
APP_PID=$!

for _ in $(seq 1 80); do
  if ! kill -0 "$APP_PID" 2>/dev/null; then
    break
  fi
  sleep 0.1
done

if kill -0 "$APP_PID" 2>/dev/null; then
  echo "foreman-overlay scripted gauntlet did not exit" >&2
  exit 1
fi

if wait "$APP_PID"; then
  :
else
  status=$?
  echo "foreman-overlay scripted gauntlet exited with status $status" >&2
  tail -80 "$APP_LOG" >&2
  exit "$status"
fi

agents_count=$(grep -Ec '^agents --json( --pull-requests)?$' "$CALL_LOG" || true)
if [ "$agents_count" -lt 2 ]; then
  echo "expected at least two agents --json calls, saw $agents_count" >&2
  cat "$CALL_LOG" >&2
  exit 1
fi
focus_count=$(grep -c '^focus --pane ' "$CALL_LOG" || true)
if [ "$focus_count" -lt 1 ]; then
  echo "expected at least one focus call from synthesized UI events, saw $focus_count" >&2
  cat "$CALL_LOG" >&2
  exit 1
fi
grep -E '^send --pane %[0-9]+ --stdin --json$' "$CALL_LOG" >/dev/null || {
  echo "missing send call" >&2
  cat "$CALL_LOG" >&2
  exit 1
}
grep -F 'overlay gauntlet send' "$STDIN_LOG" >/dev/null || {
  echo "missing expected stdin" >&2
  cat "$STDIN_LOG" >&2
  exit 1
}
for marker in \
  'overlay gauntlet flash passed' \
  'overlay gauntlet help passed' \
  'overlay gauntlet help scroll passed' \
  'overlay gauntlet arrow navigation passed' \
  'terminal activation skipped'; do
  grep -F "$marker" "$APP_LOG" >/dev/null || {
    echo "missing gauntlet marker: $marker" >&2
    cat "$APP_LOG" >&2
    exit 1
  }
done

fixture_rel="${FIXTURE#$ROOT/}"
call_log_rel="${CALL_LOG#$ROOT/}"
stdin_log_rel="${STDIN_LOG#$ROOT/}"
app_log_rel="${APP_LOG#$ROOT/}"
cat > "$SUMMARY" <<EOF
# macOS Overlay Scripted Gauntlet

- Fixture: $fixture_rel
- Fake Foreman calls: $call_log_rel
- Fake Foreman stdin: $stdin_log_rel
- Overlay log: $app_log_rel
- agents calls: $agents_count
- focus calls: $focus_count
- send stdin: passed
- flash jump via Cmd+J + label: passed
- help via ?: passed
- help scroll via j: passed
- arrow-key navigation: passed
- missing terminal activation skip: passed
- Verified: $(date -u +%Y-%m-%dT%H:%M:%SZ)
EOF

echo "macOS overlay scripted gauntlet passed"
echo "Summary: $SUMMARY"
