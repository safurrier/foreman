---
id: plan-spec
title: Help Legend And Local Navigation Trace
description: >
  Improve the help overlay so the legend is clearer and easier to find, and add
  richer local debug traces for remaining navigation lag diagnosis.
---

# Specification — help-legend-nav-trace

## Problem

Two UX gaps remain:

1. The help overlay buries the legend and does not explain the row/status glyphs well enough.
2. `j/k` feels better after the recent perf work, but remaining local lag is hard to diagnose
   because the logs do not break down enough of the action/render/inventory path.

## Requirements

### MUST

- Put a useful legend near the top of the help overlay.
- The help overlay must explain the main sidebar glyph families:
  - status glyphs
  - harness glyphs
  - session open/closed glyphs
  - shell/plain-pane marker
- Keep help/key text aligned with the current bindings and view model.
- Add richer debug traces that help diagnose local navigation lag without relying on synthetic
  perf smokes alone.
- Run the full validation ladder, including strict native E2Es.

### SHOULD

- Local debug traces should piggyback on the existing `--debug` / run-log path.
- Trace output should stay structured and grep-friendly.
- Help should stay compact enough to fit in medium and wide layouts without becoming noisy.

## Constraints

- Do not add a new heavyweight profiler or always-on verbose logs.
- Keep the trace changes off the normal hot path unless debug logging is already enabled.
- Preserve the current keybindings and sort/filter behavior.
