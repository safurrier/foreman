---
id: plan-spec
title: Stale Harness Detection And Navigation Latency
description: >
  Narrow slice for tightening compatibility-mode harness recognition and adding
  a deterministic navigation perf smoke.
---

# Specification — stale-harness-nav-latency

## Problem

Two operator-trust issues remain:

1. Compatibility recognition can keep treating a pane as Claude/Codex/Pi from stale
   title or preview text even after the foreground process has returned to a shell.
2. Navigation still feels laggy in large tmux inventories, but there is no deterministic
   perf smoke that proves repeated `j/k` selection stays within a budget under load.

## Requirements

### MUST

- Shell panes (`sh`, `bash`, `zsh`, and similar) must not be recognized as active
  Claude/Codex/Pi/Gemini/OpenCode agents from stale preview or title text alone.
- Compatibility fallback must still recognize live non-shell wrappers such as `node`
  when preview/title text identifies the harness.
- Add a deterministic profiling smoke that:
  - starts a crowded tmux fixture
  - sends a burst of `j` navigation
  - captures `move-selection` timings from Foreman logs
  - enforces a concrete latency budget
- Run the full validation ladder for the slice, including strict native E2Es.

### SHOULD

- Keep the stale-session fix narrow to harness identity, not status heuristics.
- Reuse existing runtime logging instead of adding a separate benchmarking subsystem.
- Add focused unit/integration tests for the shell-pane recognition rule.

## Constraints

- Do not regress native signal precedence.
- Do not rely on wall-clock-only assertions that are too flaky for CI.
- Keep the perf smoke at the existing `runtime_profiling` layer so it shares the
  real tmux/runtime path already used in `verify-ux` and `verify`.
