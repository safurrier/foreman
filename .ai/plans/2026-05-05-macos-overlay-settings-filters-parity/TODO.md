---
id: macos-overlay-settings-filters-parity-todo
title: TODO — macOS Overlay Settings and Filters Parity
---

# TODO

## Phase 0 — Design / decisions

- [ ] Decide if default popup size should be user-controlled only from Settings
      or also persisted from direct window resize.
- [ ] Decide whether the overlay should remain a fixed command-palette panel or
      become resizable by default.
- [ ] Decide whether Settings should be one scrolling form or tabbed sections.
- [ ] Decide how far to go on notification parity now vs placeholder.

## Phase 1 — Settings model

- [x] Add `OverlayPreferences` or similar in `ForemanOverlayCore`.
- [x] Persist preferences with UserDefaults:
  - [x] popup width
  - [x] popup height
  - [x] include all panes
  - [x] sort mode
  - [x] harness filter
  - [x] status filter
  - [x] terminal activation mode
  - [x] custom terminal bundle id
- [x] Add tests for defaults, bounds/clamping, and round-trip persistence.

## Phase 2 — Popup size

- [x] Change panel construction to read persisted size.
- [x] Add min/max constraints.
- [x] Consider making panel resizable and persisting user resize.
- [x] Add Settings controls for width/height and reset size.
- [x] Validate default size via snapshots and tests.

## Phase 3 — Settings UI

- [x] Replace current one-section Settings panel with sections:
  - [x] General
  - [x] View
  - [x] Focus
  - [x] Notifications
- [ ] General:
  - [ ] shortcut recorder
  - [ ] reset shortcut
  - [ ] popup size controls
- [ ] View:
  - [ ] include all panes toggle
  - [ ] sort picker
  - [ ] harness filter picker
  - [ ] status filter picker
- [ ] Focus:
  - [ ] terminal activation picker
  - [ ] custom bundle id text field
- [ ] Notifications:
  - [ ] show current support/placeholder honestly
  - [ ] link/copy note to future control API work if needed

## Phase 4 — Wire filters/sort to store/client

- [x] Make `ForemanClient` include `--all-panes` based on preferences.
- [x] Add store-level filtered/sorted entries:
  - [x] query filter
  - [x] harness filter
  - [x] status filter
  - [x] sort mode
- [ ] Preserve selection when filters change when possible.
- [x] Show active filter chips in footer/search area.
- [x] Add Swift unit tests for sort/filter behavior.

## Phase 5 — Search bar visual fix

- [x] Replace current oversized search row modifiers with a tighter component.
- [x] Check text/cursor vertical centering via headless snapshot.
- [x] Run headless screenshot capture.
- [ ] Run live screenshot capture if further tuning is needed.
- [x] Update OCR expectations if needed.

## Phase 6 — Detail / PR region parity

- [ ] Make active detail region more obvious without covering content.
- [ ] If PR card exists, add keyboard actions:
  - [ ] open PR
  - [ ] copy PR URL if feasible
- [ ] Add callbacks from UI/store boundary instead of direct process work in views.
- [ ] Update help/footer copy for detail/PR actions.
- [ ] Add tests/snapshots for details active with PR card.

## Phase 7 — Validation

- [x] `swift test --package-path apps/macos-overlay`
- [x] `mise run render-macos-overlay-snapshots`
- [ ] Inspect:
  - [ ] `attention.png`
  - [ ] `flash.png`
  - [ ] `compose.png`
  - [ ] details/PR active snapshot if added
- [x] `mise run verify-macos-overlay`
- [x] `mise run install-macos-overlay-app`
- [ ] Manual checks:
  - [ ] Settings sections open and persist
  - [ ] popup size applies on next open
  - [ ] filters affect visible entries
  - [ ] search cursor is vertically aligned
  - [ ] Tab to details/PR is obvious
