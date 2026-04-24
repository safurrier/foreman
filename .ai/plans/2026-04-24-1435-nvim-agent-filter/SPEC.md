# Nvim Agent Filter

## Problem

`nvim` panes can appear in agents-only mode. This is confusing because agents-only should hide editor panes unless they are actually running a supported AI harness.

## Goal

Identify why `nvim` is classified or retained as agent-visible and make agents-only filtering exclude ordinary editor panes.

## Acceptance Criteria

- Plain `nvim`/editor panes do not appear in agents-only mode.
- Actual agent panes remain visible.
- Tests cover the regression.
- `mise run check` passes before opening the PR.
