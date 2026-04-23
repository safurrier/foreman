---
id: plan-implementation
title: Implementation Plan
description: >
  Steps for sidebar virtualization.
---

# Implementation

1. Add sidebar viewport state to `AppState` and reconcile it whenever selection or visible rows
   change.
2. Add a pure helper that derives the sidebar viewport row count from terminal area/layout so the
   runtime can keep viewport state current on startup and resize.
3. Replace the sidebar `Paragraph` render path with a stateful `List` render path backed by the
   cached visible entries and viewport state.
4. Add reducer and render tests for viewport reconciliation and small-height rendering.
5. Run focused tests, then the full validation ladder.
