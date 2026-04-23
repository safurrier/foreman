---
id: plan-implementation
title: Implementation Plan
description: >
  Refresh the help overlay and add richer debug timing for local lag diagnosis.
---

# Implementation — help-legend-nav-trace

## Approach

Keep the scope narrow:
- move the legend to the top of the help overlay and rewrite the copy around the current view/sort/filter model
- add structured debug timing around inventory/tmux work and richer `move-selection` context in action timing logs

## Steps

1. Add a profiled inventory-loading path so runtime logs can emit tmux/list/capture timing summaries.
2. Extend action timing logs for `move-selection` with visible-row and viewport context.
3. Refresh the help overlay ordering and copy so the legend is first-class and bindings are accurate.
4. Add focused tests for the new help surface and trace output.
5. Run focused tests, then the full validation ladder.
