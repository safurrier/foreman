#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FOREMAN_BIN="${FOREMAN_BIN:-$ROOT/target/debug/foreman}"
KEYS="${KEYS:-20}"
TMP_DIR="$(mktemp -d)"
SOCK="$TMP_DIR/tmux.sock"

cleanup() {
  set +e
  tmux -S "$SOCK" kill-server >/dev/null 2>&1 || true
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

cargo build --quiet --manifest-path "$ROOT/Cargo.toml" --bin foreman

tmux -f /dev/null -S "$SOCK" start-server
mkdir -p "$TMP_DIR/fake-bin" "$TMP_DIR/local-work"
ln -s "$(command -v sleep)" "$TMP_DIR/fake-bin/pi"
tmux -S "$SOCK" new-session -d -s local-work "cd '$TMP_DIR/local-work' && PATH='$TMP_DIR/fake-bin':\$PATH exec pi 600"

now_ms() {
  perl -MTime::HiRes=time -e 'printf "%.0f\n", time()*1000'
}

wait_log_count() {
  log_path="$1"
  needle="$2"
  expected="$3"
  attempts="${4:-200}"
  i=0
  while [ "$i" -lt "$attempts" ]; do
    count=$(grep -a -c "$needle" "$log_path" 2>/dev/null || true)
    if [ "${count:-0}" -ge "$expected" ]; then
      return 0
    fi
    i=$((i + 1))
    sleep 0.025
  done
  echo "timed out waiting for $expected occurrences of $needle in $log_path" >&2
  return 1
}

run_burst() {
  pane="$1"
  log_path="$2"
  : > "$log_path"
  start_ms=$(now_ms)
  args=()
  i=0
  while [ "$i" -lt "$KEYS" ]; do
    args+=(j)
    i=$((i + 1))
  done
  tmux -S "$SOCK" send-keys -t "$pane" "${args[@]}"
  wait_log_count "$log_path" 'timing operation=action action=move-selection' "$KEYS" 240
  end_ms=$(now_ms)
  echo $((end_ms - start_ms))
}

run_overlap_burst() {
  pane="$1"
  log_path="$2"
  release_marker="$3"
  : > "$log_path"
  start_ms=$(now_ms)
  (
    i=0
    while [ "$i" -lt "$KEYS" ]; do
      tmux -S "$SOCK" send-keys -t "$pane" j
      i=$((i + 1))
      sleep 0.02
    done
  ) &
  sender_pid=$!
  sleep 0.3
  touch "$release_marker"
  wait_log_count "$log_path" 'timing operation=action action=move-selection' "$KEYS" 240
  wait "$sender_pid"
  end_ms=$(now_ms)
  echo $((end_ms - start_ms))
}

metric_action_count() {
  grep -a -c 'timing operation=action action=move-selection' "$1" 2>/dev/null || true
}

metric_slow_actions() {
  grep -a -c 'slow_operation operation=action threshold_ms=40 action=move-selection' "$1" 2>/dev/null || true
}

metric_render_max() {
  grep -a 'timing operation=render_frame' "$1" 2>/dev/null \
    | sed -n 's/.*elapsed_ms=\([0-9][0-9]*\).*/\1/p' \
    | sort -n \
    | tail -1
}

metric_deferred() {
  grep -a -c 'inventory_refresh_deferred generation=' "$1" 2>/dev/null || true
}

metric_deferred_apply() {
  grep -a -c 'inventory_refresh_deferred_apply generation=' "$1" 2>/dev/null || true
}

assert_no_slow_actions() {
  scenario="$1"
  log_path="$2"
  slow_actions=$(metric_slow_actions "$log_path")
  if [ "${slow_actions:-0}" -ne 0 ]; then
    echo "scenario=$scenario failed: expected slow_actions=0, got $slow_actions" >&2
    return 1
  fi
}

assert_render_budget() {
  scenario="$1"
  log_path="$2"
  budget_ms="$3"
  render_max=$(metric_render_max "$log_path")
  render_max="${render_max:-0}"
  if [ "$render_max" -gt "$budget_ms" ]; then
    echo "scenario=$scenario failed: render_max_ms=$render_max > budget_ms=$budget_ms" >&2
    return 1
  fi
}

print_summary() {
  scenario="$1"
  wall_ms="$2"
  log_path="$3"
  action_count=$(metric_action_count "$log_path")
  slow_actions=$(metric_slow_actions "$log_path")
  render_max=$(metric_render_max "$log_path")
  render_max="${render_max:-0}"
  deferred=$(metric_deferred "$log_path")
  applied=$(metric_deferred_apply "$log_path")
  printf 'scenario=%s keys=%s wall_ms=%s action_count=%s slow_actions=%s render_max_ms=%s deferred=%s deferred_apply=%s\n' \
    "$scenario" "$KEYS" "$wall_ms" "${action_count:-0}" "${slow_actions:-0}" "$render_max" "${deferred:-0}" "${applied:-0}"
}

wait_alt_capture() {
  pane="$1"
  needle="$2"
  attempts="${3:-200}"
  i=0
  while [ "$i" -lt "$attempts" ]; do
    if tmux -S "$SOCK" capture-pane -ep -t "$pane" | grep -a -q "$needle"; then
      return 0
    fi
    i=$((i + 1))
    sleep 0.05
  done
  echo "timed out waiting for pane $pane to contain $needle" >&2
  tmux -S "$SOCK" capture-pane -ep -t "$pane" >&2 || true
  return 1
}

start_dashboard() {
  name="$1"
  log_dir="$2"
  shift 2
  mkdir -p "$log_dir"
  tmux -S "$SOCK" kill-session -t "$name" >/dev/null 2>&1 || true
  tmux -S "$SOCK" new-session -d -P -F '#{pane_id}' -s "$name" \
    "FOREMAN_LOG_DIR='$log_dir' '$FOREMAN_BIN' --debug --popup --no-notify --tmux-socket '$SOCK' --capture-lines 20 $*"
}

# Local-only baseline.
local_log="$TMP_DIR/logs-local"
local_pane=$(start_dashboard dashboard-local "$local_log" --sources local --poll-interval-ms 60000)
wait_log_count "$local_log/latest.log" 'timing operation=inventory_refresh' 1 240
sleep 0.1
local_wall=$(run_burst "$local_pane" "$local_log/latest.log")
assert_no_slow_actions local-only "$local_log/latest.log"
assert_render_budget local-only "$local_log/latest.log" 100
print_summary local-only "$local_wall" "$local_log/latest.log"
tmux -S "$SOCK" send-keys -t "$local_pane" q || true

# All-source with fake SSH source that has already merged.
fake_ssh="$TMP_DIR/fake-ssh"
release_marker="$TMP_DIR/release-remote"
cat > "$fake_ssh" <<EOF_FAKE
#!/bin/sh
while [ ! -f '$release_marker' ]; do sleep 0.01; done
cat <<'JSON'
{"schemaVersion":2,"generatedAtUnixMs":1,"inventory":{"totalSessions":1,"totalWindows":1,"totalPanes":1,"visibleSessions":1,"visibleWindows":1,"visiblePanes":1},"entries":[{"id":"source:local:pane:%42","sourcePaneId":"source:local:pane:%42","sourceId":"local","sourceLabel":"Local","sourceKind":"local","paneId":"%42","sessionId":"\$remote","sessionName":"0","windowId":"@remote","windowName":"zsh","title":"remote-dots","navigationTitle":"dots","harness":"pi","harnessLabel":"Pi","status":"idle","statusLabel":"IDLE","statusSource":"compatibility","integrationMode":"compatibility","isAgent":true,"currentCommand":"pi","runtimeCommand":"pi","workingDir":"/home/discord/dots","linkedRepository":null,"workspaceName":"dots","preview":"remote ready","previewProvenance":"captured","activityScore":1,"statusRank":3,"lastActivityUnixMs":1,"lastStatusChangeUnixMs":1,"activeRunCount":null,"pullRequest":null,"extensionCards":[]}],"diagnostics":[]}
JSON
EOF_FAKE
chmod +x "$fake_ssh"
config_file="$TMP_DIR/source-config.toml"
cat > "$config_file" <<EOF_CONFIG
[sources]
default_scope = "all"
query_timeout_ms = 12000

[sources.coder]
kind = "ssh"
label = "Coder"
host = "fake-coder"
foreman = "foreman"
ssh = "$fake_ssh"

[sources.coder.display]
show_label = true
label = "Coder"
EOF_CONFIG

touch "$release_marker"
all_log="$TMP_DIR/logs-all"
all_pane=$(start_dashboard dashboard-all "$all_log" --config-file "$config_file" --sources all --poll-interval-ms 60000)
wait_alt_capture "$all_pane" '4 targets' 240
sleep 1
all_wall=$(run_burst "$all_pane" "$all_log/latest.log")
assert_no_slow_actions all-source-idle "$all_log/latest.log"
assert_render_budget all-source-idle "$all_log/latest.log" 100
if [ "$all_wall" -gt $((local_wall + 250)) ]; then
  echo "scenario=all-source-idle failed: wall_ms=$all_wall exceeds local_wall=$local_wall by more than 250ms" >&2
  exit 1
fi
print_summary all-source-idle "$all_wall" "$all_log/latest.log"
tmux -S "$SOCK" send-keys -t "$all_pane" q || true

# All-source overlap: release remote while sending keys, expect deferred merge.
rm -f "$release_marker"
overlap_log="$TMP_DIR/logs-overlap"
overlap_pane=$(start_dashboard dashboard-overlap "$overlap_log" --config-file "$config_file" --sources all --poll-interval-ms 60000)
wait_alt_capture "$overlap_pane" '2 targets' 240
: > "$overlap_log/latest.log"
overlap_wall=$(run_overlap_burst "$overlap_pane" "$overlap_log/latest.log" "$release_marker")
wait_log_count "$overlap_log/latest.log" 'inventory_refresh_deferred_apply generation=' 1 240
assert_no_slow_actions all-source-overlap "$overlap_log/latest.log"
assert_render_budget all-source-overlap "$overlap_log/latest.log" 100
if [ "$(metric_deferred "$overlap_log/latest.log")" -lt 1 ] || [ "$(metric_deferred_apply "$overlap_log/latest.log")" -lt 1 ]; then
  echo "scenario=all-source-overlap failed: expected deferred merge and deferred apply" >&2
  exit 1
fi
if [ "$overlap_wall" -gt 1500 ]; then
  echo "scenario=all-source-overlap failed: wall_ms=$overlap_wall exceeds budget 1500ms" >&2
  exit 1
fi
print_summary all-source-overlap "$overlap_wall" "$overlap_log/latest.log"
tmux -S "$SOCK" send-keys -t "$overlap_pane" q || true
