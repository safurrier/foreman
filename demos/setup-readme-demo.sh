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
operator_rc="$demo_root/operator-bashrc"
config_file="$config_home/foreman/config.toml"
claude_native_dir="$log_dir/../claude-native"
codex_native_dir="$log_dir/../codex-native"
pi_native_dir="$log_dir/../pi-native"

unset NO_COLOR
export COLORTERM=truecolor
export CLICOLOR_FORCE=1

rm -rf "$demo_root"
mkdir -p "$log_dir" "$config_home/foreman" "$state_home" "$work_root"

if [[ ! -x "$repo_root/target/debug/foreman" ]]; then
  cargo build --quiet --bins
fi

cat >"$config_file" <<'TOML'
[ui]
theme = "catppuccin"
default_sort = "attention-recent"

[notifications]
enabled = false

[pull_requests]
enabled = false
TOML

cat >"$operator_rc" <<'SH'
unset NO_COLOR
export COLORTERM=truecolor
export CLICOLOR_FORCE=1
export PS1='$ '
caption() {
  printf '\033[38;5;%sm%s\033[0m\n' "$1" "$2"
}
clear
SH

tmux -f /dev/null -S "$socket" new-session -d -s __foreman_demo_setup -n setup "sleep 3600"
tmux -S "$socket" set-option -g default-terminal "screen-256color"
tmux -S "$socket" set-option -ga terminal-overrides ",xterm*:Tc,*256col*:Tc"
tmux -S "$socket" set-option -g base-index 1
tmux -S "$socket" set-window-option -g pane-base-index 1
tmux -S "$socket" set-option -g status-left "[demo] "
tmux -S "$socket" set-option -g status-right "foreman demo"
tmux -S "$socket" set-environment -gu NO_COLOR
tmux -S "$socket" set-environment -g COLORTERM truecolor
tmux -S "$socket" set-environment -g CLICOLOR_FORCE 1

shell_quote() {
  printf "%q" "$1"
}

new_real_agent_session() {
  local session="$1"
  local window="$2"
  local workdir="$3"
  local command
  command="$4"

  mkdir -p "$workdir"
  pane_id="$(tmux -S "$socket" new-session -d -P -F "#{pane_id}" -c "$workdir" -s "$session" -n "$window" "unset NO_COLOR; export COLORTERM=truecolor CLICOLOR_FORCE=1; $command")"
  tmux -S "$socket" select-pane -t "$pane_id" -T "$window"
  printf '%s\n' "$pane_id"
}

new_real_agent_window() {
  local session="$1"
  local window="$2"
  local workdir="$3"
  local command
  command="$4"

  mkdir -p "$workdir"
  pane_id="$(tmux -S "$socket" new-window -d -P -F "#{pane_id}" -t "$session:" -n "$window" -c "$workdir" "unset NO_COLOR; export COLORTERM=truecolor CLICOLOR_FORCE=1; $command")"
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

pi_agent_dir="$demo_root/pi-agent"
pi_session_dir="$demo_root/pi-sessions"
mkdir -p "$pi_agent_dir" "$pi_session_dir"

pi_command="PI_CODING_AGENT_DIR=$(shell_quote "$pi_agent_dir") PI_CODING_AGENT_SESSION_DIR=$(shell_quote "$pi_session_dir") PI_OFFLINE=1 pi --offline --no-tools --no-extensions --no-skills --no-prompt-templates --no-themes --no-context-files --session-dir $(shell_quote "$pi_session_dir")"
if [[ -x "$HOME/.npm-global/bin/codex" ]]; then
  codex_bin="$HOME/.npm-global/bin/codex"
else
  codex_bin="$(command -v codex)"
fi
codex_command="$(shell_quote "$codex_bin")"
claude_command="claude --bare"

product_pi_spec_pane="$(new_real_agent_session \
  "product-demo" \
  "pi-spec-review" \
  "$work_root/product-demo/pi-spec-review" \
  "$pi_command")"
product_pi_fixture_pane="$(new_real_agent_window \
  "product-demo" \
  "pi-fixture-check" \
  "$work_root/product-demo/pi-fixture-check" \
  "$pi_command")"
product_pi_notes_pane="$(new_real_agent_window \
  "product-demo" \
  "pi-doc-notes" \
  "$work_root/product-demo/pi-doc-notes" \
  "$pi_command")"
product_codex_ui_pane="$(new_real_agent_window \
  "product-demo" \
  "codex-ui-polish" \
  "$work_root/product-demo/codex-ui-polish" \
  "$codex_command")"
product_pi_release_pane="$(new_real_agent_window \
  "product-demo" \
  "pi-release-check" \
  "$work_root/product-demo/pi-release-check" \
  "$pi_command")"
product_codex_docs_pane="$(new_real_agent_window \
  "product-demo" \
  "codex-docs-pass" \
  "$work_root/product-demo/codex-docs-pass" \
  "$codex_command")"
review_codex_tests_pane="$(new_real_agent_session \
  "review-lab" \
  "codex-test-run" \
  "$work_root/review-lab/codex-test-run" \
  "$codex_command")"
review_claude_notes_pane="$(new_real_agent_window \
  "review-lab" \
  "claude-review-notes" \
  "$work_root/review-lab/claude-review-notes" \
  "$claude_command")"
ops_pi_scaffold_pane="$(new_real_agent_session \
  "ops-lab" \
  "pi-agent-scaffold" \
  "$work_root/ops-lab/pi-agent-scaffold" \
  "$pi_command")"
ops_codex_foreman_pane="$(new_real_agent_window \
  "ops-lab" \
  "codex-foreman-demo" \
  "$work_root/ops-lab/codex-foreman-demo" \
  "$codex_command")"

write_signal "$pi_native_dir" "$product_pi_spec_pane" "needs_attention" 140
write_signal "$pi_native_dir" "$product_pi_fixture_pane" "needs_attention" 130
write_signal "$pi_native_dir" "$product_pi_notes_pane" "idle" 70
write_signal "$codex_native_dir" "$product_codex_ui_pane" "idle" 65
write_signal "$pi_native_dir" "$product_pi_release_pane" "idle" 55
write_signal "$codex_native_dir" "$product_codex_docs_pane" "idle" 45
write_signal "$codex_native_dir" "$review_codex_tests_pane" "idle" 60
write_signal "$claude_native_dir" "$review_claude_notes_pane" "working" 95
write_signal "$pi_native_dir" "$ops_pi_scaffold_pane" "idle" 50
write_signal "$codex_native_dir" "$ops_codex_foreman_pane" "working" 110

cat >"$(dirname "$config_file")/ui-state.json" <<JSON
{
  "sort_mode": "attention-recent",
  "theme": "catppuccin",
  "filters": {
    "show_non_agent_sessions": false,
    "show_non_agent_panes": false,
    "harness": null
  },
  "collapsed_sessions": [],
  "selection": {
    "Pane": "$ops_codex_foreman_pane"
  }
}
JSON

tmux -S "$socket" new-session -d -s operator -n console -c "$repo_root" \
  "env BASH_SILENCE_DEPRECATION_WARNING=1 bash --noprofile --rcfile '$operator_rc' -i"
tmux -S "$socket" bind-key a display-popup -h 90% -w 90% -E -- \
  env -u NO_COLOR COLORTERM=truecolor CLICOLOR_FORCE=1 \
  "$repo_root/target/debug/foreman" \
  --popup \
  --no-notify \
  --tmux-socket "$socket" \
  --log-dir "$log_dir" \
  --config-file "$config_file" \
  --claude-native-dir "$claude_native_dir" \
  --codex-native-dir "$codex_native_dir" \
  --pi-native-dir "$pi_native_dir" \
  --poll-interval-ms 250 \
  --capture-lines 40

tmux -S "$socket" kill-session -t __foreman_demo_setup

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
