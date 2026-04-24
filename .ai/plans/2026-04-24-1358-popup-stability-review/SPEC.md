# Popup Stability Review

## Problem

Foreman popup mode feels unreliable during daily use:

- Startup often shows loading/drawing instead of useful cached inventory.
- Sessions, windows, and panes appear to bounce around while being monitored.
- Notifications feel unreliable or too easy to miss.
- A broader review found additional behavior gaps between `SPEC.md`, UX expectations, and implementation.

## Goals

- Make popup startup reuse recent cache long enough to be useful across frequent launches.
- Make the default target list visually stable while preserving an opt-in attention-first sort.
- Ensure popup-mode completion notifications are not suppressed just because the row is selected.
- Update product docs/spec wording to match the behavior being shipped.
- Preserve current non-popup notification semantics unless intentionally changed.
- Record follow-up bugs separately rather than sneaking broad unrelated fixes into this slice.

## Non-Goals

- Do not rewrite the inventory model or tmux adapter.
- Do not add new notification backends.
- Do not change real harness/native hook protocols.
- Do not fix every review finding in this one patch.

## Acceptance Criteria

- Popup startup cache freshness is long enough for repeated popup launches.
- Default sort is stable by session/window/pane name/title/id and does not reorder based on volatile status changes.
- `o` still exposes attention-first sorting for operators who want urgency sorting.
- Popup mode can emit completion notifications for the selected pane.
- Unit/runtime tests cover the changed behaviors.
- `mise run check` passes or any failure is understood and addressed.
