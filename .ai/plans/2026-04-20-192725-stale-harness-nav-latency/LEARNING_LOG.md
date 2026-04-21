---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises.
---

# Learning Log

## 2026-04-20 19:30 MST

- The stale-recognition root cause is compatibility identity, not native overlays.
  `recognize_harness()` currently treats `pane_title` and captured preview as enough
  to identify Claude/Codex/Pi even when `pane_current_command` is back to `zsh`.
- Real local tmux inspection showed an important nuance: Claude appears as `claude`,
  but Codex and Pi currently appear as `node` in `#{pane_current_command}`. The fix
  therefore needs a shell guard, not an exact-command-only allowlist.

## 2026-04-20 20:20 MST

- The shell guard was correct, but several synthetic tmux tests had been faking
  “agents” with long-lived `sh` loops. Those fixtures were silently relying on
  exactly the stale-shell behavior the product should reject.
- Converting those tests to non-shell Python wrappers made the suite more honest
  and also surfaced the same issue in the real Claude and Pi E2Es, where the
  dashboard actionable row depended on a shell-backed prompt loop.
- The crowded-navigation perf smoke passed with the current budgets, so the
  remaining lag reports are more likely to come from background refresh work or
  local environment pressure than from the pure `move-selection` path alone.
