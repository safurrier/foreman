---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for this unit of work.
  Optional — create only when the approach isn't obvious.
---

# Implementation — startup-loading-flash-hint

## Approach

- Start runtime with a minimal `AppState` marked as `startup_loading`.
- Paint once immediately after terminal setup, then request the first tmux inventory refresh asynchronously.
- Use a lighter startup preview policy so initial refresh does not force full capture for every pane.
- Replace the flash popup overlay with inline row labels plus a smaller footer hint.
- Re-run focused startup/flash tests first, then the full validation ladder, then strict native verification.

## Steps

1. Add explicit startup-loading state and runtime bootstrap support for a first-paint-before-inventory flow.
2. Route initial inventory through the async refresh worker and clear loading state on success or failure.
3. Stage startup preview capture so unseen and empty previews do not block first paint.
4. Remove the flash popup overlay and move guidance into inline/footer UI.
5. Update runtime smokes, release gauntlet assertions, profiling smokes, and real Claude/Pi E2Es to match the new timing model.
6. Run `mise run check`, `mise run verify`, `mise run native-preflight`, and strict `mise run verify-native`.
