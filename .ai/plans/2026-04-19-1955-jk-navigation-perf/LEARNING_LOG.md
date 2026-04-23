---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary for this slice.
---

# Learning Log

## 2026-04-19 19:55 MST

- The slow feeling on `j` and `k` is not coming from tmux focus commands. `MoveSelection` is reducer-only, but it rebuilds `visible_targets()` on each keypress and render repeats similar work immediately after.
- Pull request lookups are a secondary concern here, not the primary hot path. The runtime already debounces selection-driven PR lookups and the existing perf smoke proves that settled-selection behavior.
- The most suspicious render cost is `sidebar_line`, which recomputes visible window/pane summaries for each session/window row on every frame instead of reusing a derived structure.

## 2026-04-19 22:41 MST

- The right narrow fix was a derived visible-state cache, not a speculative async change. Caching `SelectionTarget` order, sidebar row metadata, workspace paths, actionable pane ids, and flash targets removed the main repeated work from both reducer navigation and render.
- The cache needs to be treated like any other derived state: every inventory replacement, visibility toggle, sort change, search edit, and session collapse/expand path has to rebuild it before selection reconciliation. Missing even one of those paths would have created stale selection bugs that are harder to diagnose than the original lag.
- This slice improved the hot path without changing the rendering primitive. The sidebar still renders as a full `Paragraph`, so if the operator still feels lag in very large trees the next step is likely true list virtualization rather than another round of micro-caching.
- The existing validation ladder was already strong enough for this refactor. `runtime_profiling`, `runtime_dashboard`, `release_gauntlet`, `mise run verify`, and strict `verify-native` together gave enough coverage to change a central state path with confidence.

## Retrospective

- Matched plan:
  - Added cached visible/sidebar state to `AppState`
  - Switched reducer navigation and selection reconciliation onto the cached entries
  - Switched sidebar rendering and several preview lookups onto cached row metadata
  - Ran the full validation ladder, including strict real native E2Es
- Diverged from plan:
  - No new dedicated perf budget assertion was added in this slice. Existing runtime profiling and dashboard smoke coverage were enough for this change, but a future slice could still add a tighter navigation-frame budget if we want regression enforcement at the millisecond level.
- What would make this more one-shot next time:
  - A documented checklist of every reducer branch that mutates visibility/search/sort/collapse state would make derived-cache refactors less audit-heavy.
  - If we decide to pursue a second perf pass, we should go straight to sidebar virtualization instead of trying another incremental render micro-optimization.
