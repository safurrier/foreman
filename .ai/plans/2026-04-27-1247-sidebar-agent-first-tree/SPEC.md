# SPEC — Sidebar Agent-First Tree

## Problem

The default sidebar should optimize for the operator's semantic model: sessions contain actionable agents. Raw tmux windows are useful topology, but singleton window rows add redundant nesting in the normal agents-only view.

## Requirements

- Default agents-only view renders `session -> agent pane` for singleton agent windows.
- Pane rows carry the semantic harness/status signal directly through glyph and styling.
- Topology/all-target modes keep explicit `session -> window -> pane` rows.
- Multi-pane windows remain explicit so users can distinguish panes under one tmux window.
- Popup and regular modes remain visually consistent.
- Regression tests and UX artifacts cover the behavior.

## Non-Goals

- No tmux inventory discovery changes.
- No new user-facing toggle beyond existing visibility/topology filters.
