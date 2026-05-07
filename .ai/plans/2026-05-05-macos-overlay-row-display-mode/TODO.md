---
id: macos-overlay-row-display-mode-todo
title: TODO — macOS Overlay Row Display Mode
---

# TODO

## Phase 1 — Model / preferences

- [ ] Add `OverlayRowDisplayMode` enum.
- [ ] Persist row display mode in `OverlayPreferences`.
- [ ] Add Settings → View picker for row title mode.

## Phase 2 — Display title logic

- [ ] Add row title/subtitle helper in Core or UI.
- [ ] Implement smart duplicate detection over current visible entries.
- [ ] Treat generic/empty window names as low-value.
- [ ] Include pane id/workspace in subtitle when needed.

## Phase 3 — Fixtures / snapshots

- [ ] Add fixture with duplicate workspace entries but distinct window names.
- [ ] Add snapshot state `duplicate-workspace`.
- [ ] Update OCR expectations.

## Phase 4 — Tests

- [ ] Unit test smart display mode with duplicates.
- [ ] Unit test explicit workspace/window/session/pane title modes.
- [ ] Run Swift tests.

## Phase 5 — Validation

- [ ] `swift test --package-path apps/macos-overlay`
- [ ] `mise run render-macos-overlay-snapshots`
- [ ] inspect duplicate-workspace snapshot
- [ ] `mise run verify-macos-overlay`
- [ ] `mise run install-macos-overlay-app`
