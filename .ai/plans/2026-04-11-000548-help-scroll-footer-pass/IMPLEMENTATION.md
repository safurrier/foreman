---
id: plan-implementation
title: Implementation Plan
description: >
  Ordered implementation steps for this unit of work.
---

# Implementation — help-scroll-footer-pass

1. Add reducer-owned help view state.
   - Extend `AppState` with a help scroll offset.
   - Add explicit actions for help scrolling and reset.
   - Route `j/k`, arrows, PageUp/PageDown, Home, and End while help is open.

2. Rebuild the help popup as a bounded reference surface.
   - Use the scroll offset in the renderer.
   - Add a visible hint that the popup is scrollable and how to close it.
   - Keep the legend and focus explanation intact, but structure the content so
     important sections are near the top.

3. Rework the footer language.
   - Replace compressed shorthand with grouped phrases such as `Enter act`,
     `f jump tmux`, `i compose`, and `? help`.
   - Make compact/medium/wide variants focus-aware so the footer surfaces the
     controls most relevant to the active pane and current mode.

4. Strengthen validation.
   - Add render tests for the scrollable help surface.
   - Extend the tmux-backed runtime walkthrough to prove help scrolling on a
     constrained window size.
   - Re-run `verify-ux`, `check`, and `verify`.

5. Sync the docs and plan trail.
