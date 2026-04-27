---
id: plan-spec
title: Task Specification
description: Requirements and constraints for sidebar scanability and singleton-window elision.
---

# SPEC — sidebar-scanability-singletons

## Problem

The sidebar currently exposes raw tmux topology in normal agent view: session -> window -> pane. In common workspaces with one window and one pane, the window row duplicates the pane/worktree name and pushes useful agent rows farther apart. The screenshot review showed this makes the left side harder to scan than it needs to be.

## Requirements

- Preserve exact tmux topology in topology/debug scenarios where window rows add value.
- In the normal agent-first view, elide a window row when it has exactly one visible pane and the window itself has no extra structure to communicate.
- Keep selection, Enter behavior, flash labels, focus, direct input, and kill/rename target resolution correct when the visible row points directly to the pane.
- Avoid showing redundant `1w/1p` and `1p` counts when singleton elision already communicates that structure.
- Keep no-color behavior and existing theme glyphs.
- Update SPEC and tests to document the default operator view versus topology view distinction.
- Run screenshot/GIF UX validation before pushing. `mise run verify-ux` is the mid/heavy visual gate, not part of the fast gate.

## Non-goals

- New persistence/config toggles for sidebar density.
- New data model for tasks/worktrees.
- Rewriting the sidebar into grouped attention sections.

## Acceptance

- Render tests show singleton windows collapse in agent-first view while multi-pane windows still render.
- Runtime/release tests prove focus/navigation still resolve the correct tmux pane.
- `mise run check` passes.
- `mise run verify-ux` passes and refreshes screenshots/GIF before PR push.
- CI `Quality Gate` and `Full Validation` pass after push.
