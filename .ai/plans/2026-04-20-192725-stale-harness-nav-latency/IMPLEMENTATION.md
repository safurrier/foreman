---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for stale harness detection and navigation latency coverage.
---

# Implementation — stale-harness-nav-latency

## Approach

Treat the live tmux foreground command as the first guard for compatibility identity:
if the pane is back at a shell, stale preview text no longer revives an agent row.
For perf, use the existing debug timing logs from the runtime loop and add a heavier
`runtime_profiling` smoke that asserts repeated `move-selection` actions stay under
a budget with many visible rows.

## Steps

1. Add a shell-command guard to compatibility harness recognition and cover it with
   integration/unit tests.
2. Update tmux adapter tests so compatibility recognition fixtures model live
   non-shell wrappers instead of shell prompts.
3. Add a new ignored `runtime_profiling` smoke that bursts `j` navigation across
   a large tmux fixture, parses `move-selection` timings, and enforces a latency budget.
4. Run focused tests first, then the full validation ladder:
   `mise run check`, `mise run verify`, `mise run native-preflight`, and strict
   `mise run verify-native`.
