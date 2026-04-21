---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary for this slice.
---

# Learning Log

## 2026-04-20 10:17 MST

- The previous cache slice removed most repeated sorting and row-derivation work, but the sidebar
  is still rendered as one `Paragraph` over all rows.
- Ratatui's `List` widget is actually useful here because its stateful render path only iterates
  the visible item range. That makes it a better fit than hand-slicing `Paragraph` text for this
  specific single-line row model.
- The main design pressure is preserving the codebase's mostly-pure render path while still
  keeping viewport state stable. The least invasive compromise is to store sidebar viewport state
  in `AppState` and sync viewport height from runtime startup/resize events.

## 2026-04-20 12:48 MST

- The first implementation worked in unit coverage but regressed the release gauntlet because tmux
  did not reliably surface a resize event after the dashboard pane was resized. Relying on startup
  plus `Event::Resize` was not enough.
- The robust fix was to reconcile the sidebar viewport row count against `terminal.size()` at the
  top of the runtime loop before each draw. That kept render pure while making viewport state track
  the actual terminal size even when resize events were dropped or delayed.
- Ratatui `List` virtualization is a good fit here because rows are already one line. If the
  sidebar ever becomes multi-line or heavily wrapped, the next iteration would need a more explicit
  row-height model.

## Retrospective

- Matched plan:
  - Added sidebar viewport state and reconciliation
  - Added a shared viewport-size helper for runtime/layout sync
  - Replaced the sidebar `Paragraph` path with a virtualized stateful list
  - Added focused reducer/render coverage and ran the full validation ladder
- Diverged from plan:
  - The first runtime integration assumed resize events were reliable enough for viewport sync.
    The release gauntlet disproved that and forced one more runtime pass.
- What would make this more one-shot next time:
  - Any TUI state that depends on terminal geometry should be treated as live runtime state, not
    event-only state. The gauntlet caught that quickly once it exercised tmux resizing.
