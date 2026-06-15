#!/usr/bin/env bash
set -euo pipefail

cargo test --test pi_subagent_agent_sim -- --nocapture
cargo test -q pi_subagents
