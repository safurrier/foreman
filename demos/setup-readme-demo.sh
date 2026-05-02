#!/usr/bin/env bash
set -euo pipefail

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  echo "source this script so it can export FOREMAN_DEMO_* variables" >&2
  exit 1
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
demo_root="${FOREMAN_DEMO_ROOT:-/tmp/foreman-readme-demo}"
socket="$demo_root/tmux.sock"
log_dir="$demo_root/logs"
config_home="$demo_root/config"
state_home="$demo_root/state"
work_root="$demo_root/work"
runner="$demo_root/agent-runner.sh"
config_file="$config_home/foreman/config.toml"
claude_native_dir="$log_dir/../claude-native"
codex_native_dir="$log_dir/../codex-native"
pi_native_dir="$log_dir/../pi-native"

rm -rf "$demo_root"
mkdir -p "$log_dir" "$config_home/foreman" "$state_home" "$work_root"

if [[ ! -x "$repo_root/target/debug/foreman" ]]; then
  cargo build --quiet --bins
fi

cat >"$config_file" <<'TOML'
[ui]
theme = "catppuccin"
default_sort = "stable"

[notifications]
enabled = false
TOML

cat >"$runner" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

cd "$1"
banner="$2"
prefix="$3"

printf '%s\n' "$banner"
while IFS= read -r line; do
  printf '%s:%s\n' "$prefix" "$line"
done
SH
chmod +x "$runner"

tmux -f /dev/null -S "$socket" start-server

shell_quote() {
  printf "%q" "$1"
}

new_agent_session() {
  local session="$1"
  local window="$2"
  local workdir="$3"
  local banner="$4"
  local prefix="$5"
  local command

  mkdir -p "$workdir"
  command="$runner $(shell_quote "$workdir") $(shell_quote "$banner") $(shell_quote "$prefix")"
  pane_id="$(tmux -S "$socket" new-session -d -P -F "#{pane_id}" -c "$workdir" -s "$session" "$command")"
  tmux -S "$socket" rename-window -t "$session:0" "$window"
  tmux -S "$socket" select-pane -t "$pane_id" -T "$window"
  printf '%s\n' "$pane_id"
}

write_signal() {
  local native_dir="$1"
  local pane_id="$2"
  local status="$3"
  local score="$4"

  mkdir -p "$native_dir"
  printf '{"status":"%s","activity_score":%s}' "$status" "$score" >"$native_dir/$pane_id.json"
}

claude_pane="$(new_agent_session \
  "claude-review" \
  "website" \
  "$work_root/website" \
  "Claude Code needs approval for the design patch" \
  "CLAUDE")"
codex_pane="$(new_agent_session \
  "codex-refactor" \
  "native-hooks" \
  "$work_root/native-hooks" \
  "Codex is editing the native hook reducer" \
  "CODEX")"
pi_pane="$(new_agent_session \
  "pi-dots" \
  "dots" \
  "$work_root/pi-dots" \
  "Pi is checking dotfile setup output" \
  "PI")"

write_signal "$claude_native_dir" "$claude_pane" "needs_attention" 95
write_signal "$codex_native_dir" "$codex_pane" "working" 120
write_signal "$pi_native_dir" "$pi_pane" "idle" 40

export FOREMAN_CONFIG_HOME="$config_home"
export XDG_STATE_HOME="$state_home"
export FOREMAN_DEMO_ROOT="$demo_root"
export FOREMAN_DEMO_SOCKET="$socket"
export FOREMAN_DEMO_LOG_DIR="$log_dir"
export FOREMAN_DEMO_CONFIG="$config_file"
export FOREMAN_DEMO_CLAUDE_NATIVE_DIR="$claude_native_dir"
export FOREMAN_DEMO_CODEX_NATIVE_DIR="$codex_native_dir"
export FOREMAN_DEMO_PI_NATIVE_DIR="$pi_native_dir"

echo "Foreman README demo tmux world ready: $demo_root"
