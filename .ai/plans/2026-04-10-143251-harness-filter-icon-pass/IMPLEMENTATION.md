---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for this unit of work.
  Optional — create only when the approach isn't obvious.
---

# Implementation — harness-filter-icon-pass

## Approach

Treat this as an information-architecture pass, not a theme pass. The key
technical decision is to avoid real brand logos and instead use semantic harness
marks with a legend and ASCII fallback. Actual image/logo rendering is too
terminal-specific for a tmux-first Ratatui app.

The work stays in four slices:
1. state and command support for a harness filter
2. render updates for clearer row grammar, harness marks, and legend/help
3. live runtime validation for filter/help flow
4. doc/spec/plan/artifact sync

## Steps

1. Extend `Filters`, `HarnessKind`, command mapping, and reducer support for a
   keyboard-cycled harness filter.
2. Update visibility rules so harness filters apply consistently to panes,
   windows, sessions, and actionable-pane selection.
3. Replace opaque harness text badges in the renderer with compact visual marks
   plus theme-backed ASCII fallback.
4. Tighten the sidebar, preview, help, and footer so act-on-selection intent is
   explicit and the active filter is obvious.
5. Add reducer tests, render tests, and a live tmux runtime smoke covering help
   and harness filter cycling.
6. Refresh `verify-ux` artifacts if needed and sync the top-level spec/docs plus
   the work-unit logs.
