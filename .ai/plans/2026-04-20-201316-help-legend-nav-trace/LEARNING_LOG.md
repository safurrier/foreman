---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises.
---

# Learning Log

## 2026-04-20 20:15 MST

- The current help overlay already contains the right raw information, but the order is wrong
  for first use: `Legend` comes after `Source` and `Navigate`, so the operator has to scroll
  before they can decode the sidebar.
- The runtime already emits `action`, `render_frame`, `inventory_refresh`, and PR lookup timing,
  so the cheapest way to improve local diagnosis is to enrich those logs rather than add a
  separate tracing system.

## 2026-04-20 22:56 MST

- The first pass only logged `inventory_tmux` and `inventory_native` during later refreshes.
  The crowded-navigation perf smoke caught that gap because short-lived dashboard runs never hit
  a refresh cycle. Logging the same timing breakdown during bootstrap fixed that blind spot.
- Reordering the help surface broke the small-layout runtime smoke because it still expected
  `Navigate` to be the first visible section. The regression was in the test contract, not the
  product. Updating the tmux smoke to expect `Legend` first kept the validation honest.
- Render-level assertions for long wrapped help text were brittle. It was more stable to assert
  the debug help content from `help_lines(...)` directly and reserve runtime smokes for the live
  scroll behavior.

## Retrospective

- Matched plan:
  - Added richer local timing traces without inventing a second tracing system.
  - Moved the legend to the top of the help overlay and made the glyph meanings clearer.
  - Added focused coverage and reran the full validation ladder including strict real native E2E.
- Diverged from plan:
  - Needed one extra bootstrap logging pass after the perf smoke exposed the missing inventory
    timing lines in short runs.
  - Needed one runtime smoke expectation update because the help overlay ordering changed.
- One-shot improvement next time:
  - When changing help ordering, search the tmux runtime and release gauntlet tests up front for
    hard-coded section names before running the full suite.
