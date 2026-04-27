---
id: plan-spec
title: Task Specification
description: Requirements and constraints for the Details pane scan-zone polish pass.
---

# SPEC — details-pane-scan-zones

## Problem

The Details pane now has named sections, but too much of the content still reads like loose prose. Operators should be able to learn fixed scan positions: current target/status, repo/worktree, PR, diagnostics, overview, and terminal output.

Research and prior plans point to a dashboard-style pattern: stable zones, aligned key/value rows, semantic color for status/source/PR/diagnostics, and no-color glyph fallbacks. The slice must improve scanability without adding runtime state or consuming much more vertical space.

## Requirements

- Keep `render = pure(state)`; no runtime side effects or new services.
- Keep existing section order and compact/no-color behavior.
- Make section headers and important rows visually more consistent.
- Use aligned labels and stable labels for recurring facts such as status, source, target, workspace, PR, diagnostics, and overview.
- Use semantic styles for values that already carry state: status, source confidence, PR state, diagnostics, cache/activity, and preview provenance.
- Avoid hard-coded Unicode that breaks `no-color`.
- Update root `SPEC.md` if the visible Details contract changes.

## Non-goals

- New panes, tabs, or navigation modes.
- New data sources for git, branch, or PR state.
- A full cockpit/layout rewrite.

## Acceptance

- Render tests prove the structured labels/headers and no-color fallback.
- Runtime dashboard/release gauntlet tests still pass.
- `mise run check` passes.
- UX artifacts are refreshed if visual output changes.
