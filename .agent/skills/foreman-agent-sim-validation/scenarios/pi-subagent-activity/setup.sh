#!/usr/bin/env bash
set -euo pipefail

# No persistent setup is required. The Rust test creates isolated tmux sockets,
# temp workspaces, and synthetic Pi subagent status files.
command -v tmux >/dev/null
cargo test --test pi_subagent_agent_sim --no-run
