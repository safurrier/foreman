#!/bin/sh
set -eu

# Opt-in only: this smoke changes the current Ghostty terminal title briefly and
# asks macOS Automation to focus that exact terminal. It never queries or
# mutates the operator's default tmux server.
if [ "${FOREMAN_GHOSTTY_DISPLAY_SMOKE:-0}" != "1" ]; then
  printf '%s\n' 'SKIP: set FOREMAN_GHOSTTY_DISPLAY_SMOKE=1 inside the Ghostty terminal that may be focused.'
  exit 0
fi

if [ "$(uname -s)" != "Darwin" ]; then
  printf '%s\n' 'ERROR: the Ghostty display smoke requires macOS.' >&2
  exit 1
fi

case "${TERM_PROGRAM:-}" in
  ghostty|Ghostty) ;;
  *)
    printf '%s\n' 'ERROR: run this opt-in smoke from the Ghostty terminal that Foreman may rename and focus.' >&2
    exit 1
    ;;
esac

if ! command -v tmux >/dev/null 2>&1; then
  printf '%s\n' 'ERROR: tmux is required for the isolated focus half of this smoke.' >&2
  exit 1
fi

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TMP_ROOT=$(mktemp -d "${TMPDIR:-/tmp}/foreman-ghostty-display.XXXXXX")
SERVER="foreman-ghostty-display-$$"
CONFIG_FILE="$TMP_ROOT/config.toml"
LOG_DIR="$TMP_ROOT/state/logs"
CAPTURE_JSON="$TMP_ROOT/capture.json"
FOCUS_JSON="$TMP_ROOT/focus.json"
ORIGINAL_TITLE=''
OWNERSHIP_HANDLE=''

if [ -n "${FOREMAN_BIN:-}" ]; then
  BIN=$FOREMAN_BIN
else
  (cd "$ROOT" && cargo build --quiet --bin foreman)
  BIN="$ROOT/target/debug/foreman"
fi

cleanup() {
  if [ -n "$OWNERSHIP_HANDLE" ]; then
    "$BIN" --config-file "$CONFIG_FILE" --log-dir "$LOG_DIR" \
      sources display unregister local --handle "$OWNERSHIP_HANDLE" --json \
      >/dev/null 2>&1 || true
  fi
  tmux -L "$SERVER" kill-server >/dev/null 2>&1 || true
  if [ -n "$ORIGINAL_TITLE" ]; then
    printf '\033]0;%s\007' "$ORIGINAL_TITLE"
  fi
  rm -rf "$TMP_ROOT"
}
trap cleanup EXIT HUP INT TERM

"$BIN" --config-file "$CONFIG_FILE" --log-dir "$LOG_DIR" \
  sources display capture local --provider ghostty --json >"$CAPTURE_JSON"

OWNERSHIP_HANDLE=$(python3 - "$CAPTURE_JSON" <<'PY'
import json, sys
value = json.load(open(sys.argv[1]))
print(value["registration"]["ownershipHandle"])
PY
)
ORIGINAL_TITLE=$(python3 - "$CAPTURE_JSON" <<'PY'
import json, sys
value = json.load(open(sys.argv[1]))
print(value["registration"]["identity"].get("diagnosticTitle") or "Foreman Ghostty")
PY
)
TERMINAL_UUID=$(python3 - "$CAPTURE_JSON" <<'PY'
import json, sys
value = json.load(open(sys.argv[1]))
print(value["registration"]["identity"]["terminalUuid"])
PY
)

RENAMED_TITLE="Foreman exact-ID smoke $$"
printf '\033]0;%s\007' "$RENAMED_TITLE"
sleep 0.2

# This server name is passed to every tmux invocation. The default server is
# never inspected, focused, or killed.
tmux -L "$SERVER" new-session -d -s display-smoke "sh -c 'exec sleep 30'"
PANE_ID=$(tmux -L "$SERVER" display-message -p -t display-smoke '#{pane_id}')

"$BIN" --config-file "$CONFIG_FILE" --log-dir "$LOG_DIR" \
  --tmux-server-name "$SERVER" focus --pane "$PANE_ID" --json >"$FOCUS_JSON"

python3 - "$CAPTURE_JSON" "$FOCUS_JSON" "$TERMINAL_UUID" "$RENAMED_TITLE" <<'PY'
import json, sys
capture = json.load(open(sys.argv[1]))
focus = json.load(open(sys.argv[2]))
expected_uuid = sys.argv[3]
renamed_title = sys.argv[4]
assert capture["registration"]["identity"]["terminalUuid"] == expected_uuid
assert focus["ok"] is True, focus
activation = focus.get("callerDisplayActivation")
assert activation and activation["attempted"] is True and activation["ok"] is True, focus
assert activation["provider"] == "ghostty", activation
print(json.dumps({
    "ok": True,
    "action": "smoke.ghostty-display-registration",
    "terminalUuid": expected_uuid,
    "renamedTitle": renamed_title,
    "tmuxServer": "isolated",
    "callerDisplayActivation": activation,
}, indent=2))
PY
