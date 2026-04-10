---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for this unit of work.
  Optional — create only when the approach isn't obvious.
---

# Implementation — sidebar-density-pass

## Approach

Treat this as one interaction-quality slice with two threads:

1. correctness/performance:
   decouple selection movement from synchronous PR lookup, add actionable-pane
   resolution for focus/compose, and emit runtime timing telemetry to logs
2. information architecture:
   trim idle copy, add a legend, and push dense reference material into help instead
   of the footer

The live tmux harness remains the source of truth for operator behavior. Render tests
cover static density changes; a scripted tmux smoke covers the lag regression.

## Steps

1. Fill the work unit with concrete findings from the live walkthrough and repro.
2. Add runtime timing/profiling logs and slow-operation warnings on hot paths.
3. Remove PR lookup from immediate navigation and replace it with a deferred refresh
   path that runs after the user stops moving.
4. Add actionable-pane resolution for focus, compose, and kill flows from
   session/window selections.
5. Tighten render output:
   denser sidebar rows, help legend, quieter footer, clearer compose hints.
6. Update key handling so `Enter` sends in input mode and newline uses the
   documented alternate shortcut.
7. Add heavy validation:
   live tmux perf smoke, log assertions, and UX verification updates.
8. Sync docs/spec/plan validation artifacts.
