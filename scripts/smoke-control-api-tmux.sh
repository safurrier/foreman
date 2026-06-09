#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FOREMAN_BIN="${FOREMAN_BIN:-$ROOT/target/debug/foreman}"
TMP_DIR="$(mktemp -d)"
SOCK="$TMP_DIR/tmux.sock"
SESSION="foreman-control-smoke"

cleanup() {
  set +e
  tmux -S "$SOCK" kill-server >/dev/null 2>&1 || true
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

cargo build --quiet --manifest-path "$ROOT/Cargo.toml" --bin foreman

ln -s "$(command -v sleep)" "$TMP_DIR/claude"
tmux -S "$SOCK" new-session -d -s "$SESSION" -n agent "env PATH=$TMP_DIR:\$PATH sh -lc 'claude 30'"
PANE="$(tmux -S "$SOCK" list-panes -a -F '#{pane_id}' | head -n1)"
tmux -S "$SOCK" select-pane -t "$PANE" -T claude
sleep 0.2

AGENTS_JSON="$TMP_DIR/agents.json"
"$FOREMAN_BIN" --tmux-socket "$SOCK" agents --json > "$AGENTS_JSON"
python3 - "$AGENTS_JSON" "$PANE" <<'PY'
import json, sys
path, pane = sys.argv[1], sys.argv[2]
payload = json.load(open(path))
assert payload["schemaVersion"] == 2, payload
entries = payload["entries"]
assert any(entry["paneId"] == pane for entry in entries), payload
entry = next(entry for entry in entries if entry["paneId"] == pane)
assert entry["harness"] == "claude-code", entry
print(f"agents ok: {pane} {entry['harness']} {entry['status']}")
PY

"$FOREMAN_BIN" --tmux-socket "$SOCK" focus --pane "$PANE" --json | python3 -m json.tool >/dev/null
printf 'printf overlay_send_ok' | "$FOREMAN_BIN" --tmux-socket "$SOCK" send --pane "$PANE" --stdin --json | python3 -m json.tool >/dev/null
sleep 0.5
CAPTURE="$(tmux -S "$SOCK" capture-pane -p -t "$PANE" -S -20)"
printf '%s\n' "$CAPTURE" | grep -F 'overlay_send_ok' >/dev/null

echo "control API real tmux smoke passed for pane $PANE"
