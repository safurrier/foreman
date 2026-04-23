---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
---

# Specification — help-scroll-footer-pass

## Problem

The dashboard is materially better than the earlier cuts, but two operator
frictions remain obvious in real use:

- the footer is now compact enough to fit, but some lines read like glyph soup
  or shorthand that requires memorization instead of glanceable understanding
- the help popup is a fixed paragraph, so on smaller terminals it clips instead
  of behaving like a real reference surface

This slice should improve comprehension without bloating the steady-state UI or
breaking the fast keyboard workflow.

## Requirements

### MUST

- Normal-mode footer copy uses a middle ground between terse mnemonics and
  verbose prose. It must remain glanceable, but the control groups must read as
  actions rather than random letters.
- The help popup becomes scrollable with keyboard controls that fit the existing
  navigation model.
- Help scrolling is reducer-owned state, not widget-local mutation.
- The help surface explicitly explains harness marks, status marks, and how tmux
  focus resolves to the actionable pane.
- Smaller terminal layouts keep the help surface usable instead of silently
  clipping critical sections.
- Live tmux-backed validation proves the scroll behavior and the revised footer
  language.
- Repo truth stays synced:
  - `SPEC.md`
  - `README.md`
  - `docs/architecture.md`
  - `docs/workflows.md`
  - this plan trail

### SHOULD

- The footer should adapt by focus and mode so the operator sees the next
  sensible actions, not the whole product command surface at once.
- Help should expose its own scroll affordance near the footer or popup frame.
- The new copy should reduce unexplained abbreviations in the steady-state UI.

## Constraints

- Keep normal keyboard navigation instant. Help scrolling should not introduce
  animation or delayed behavior.
- Do not make the footer taller in normal mode.
- Do not turn the help popup into a separate screen with a different event loop.
- Preserve monochrome and ASCII-safe behavior.
