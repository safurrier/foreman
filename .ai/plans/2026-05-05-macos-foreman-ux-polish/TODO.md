---
id: macos-foreman-ux-polish-todo
title: TODO — macOS Foreman UX Polish
---

# TODO

## Phase 1 — Low-risk copy/naming

- [x] Rename app bundle output from `Foreman Overlay.app` to `Foreman.app`.
- [x] Change bundle display/name strings to `Foreman`.
- [x] Consider bundle id transition:
  - [x] migrate to `dev.foreman.app` and unregister old bundle path.
- [x] Update install script to remove/unregister old `~/Applications/Foreman Overlay.app`.
- [x] Update launch docs to use `open -a Foreman`.
- [x] Update menu titles/copy:
  - [x] `About Foreman`
  - [x] `Quit Foreman`
  - [x] `Open Foreman`
  - [x] Settings window title `Foreman Settings`
- [x] Change search placeholder to `Search Agent Sessions`.
- [x] Update OCR semantic expectations.

## Phase 2 — Settings visibility

- [x] Make `SettingsPanelController` use a window level above the overlay, or
      hide/demote the overlay before showing settings.
- [x] Use `NSApp.activate(ignoringOtherApps: true)` and `orderFrontRegardless()`
      for Settings after the panel content is installed.
- [ ] Add a log/test-mode signal for Settings open if practical.
- [ ] Manual proof: footer Settings, status menu Settings, app menu Settings,
      and Cmd+, all visibly open the same panel.

## Phase 3 — Remove content-covering badges

- [x] Remove `RegionBadge` overlays from `List`, detail preview, and compose
      text field.
- [x] Replace with layout-safe focus treatment:
  - [x] `ActiveRegionIndicator` in footer or panel header
  - [x] active panel border/ring using `.overlay` on containers, not controls
  - [x] accessible labels for active region
- [x] Ensure selected row status text remains unobscured.
- [x] Ensure compose input placeholder and send affordance remain unobscured.

## Phase 4 — Help modality / scroll isolation

- [x] When help is visible, disable underlying `List`/preview scroll views with
      `.scrollDisabled(store.isHelpVisible)` where supported.
- [x] Give Help overlay a full-frame hit-test layer/content shape to intercept
      pointer and scroll events.
- [x] Keep help's own `ScrollView` scrollable.
- [ ] Add/adjust UI-event gauntlet to prove arrow/j/k help scrolling does not
      change agent selection.
- [ ] Manual proof: trackpad scroll over Help does not move the agent list.

## Phase 5 — Cmd+T without system beep

- [x] Add a real app/menu command: `Cycle Theme`, key equivalent Cmd+T.
- [x] Route menu command through `store.cycleTheme()`.
- [x] Ensure local monitor and menu command do not double-cycle.
- [ ] Manual proof: Cmd+T cycles once and does not play invalid-command sound.

## Phase 6 — Styling primitives / regression snapshots

- [x] Add lightweight SwiftUI primitives/constants rather than ad hoc overlays:
  - [ ] `OverlayStyle` / token constants
  - [x] active panel style
  - [x] footer command chip style
  - [x] compose input row style
- [x] Add or update snapshot fixtures/states for:
  - [x] list active
  - [x] details active
  - [x] compose active
  - [x] help open over list
- [x] Run `mise run render-macos-overlay-snapshots` and inspect PNGs.
- [x] Run `mise run verify-macos-overlay`.

## Deferred

- [ ] App icon polish.
- [ ] Launch-at-login setting.
- [ ] Deeper hyperkey debugging unless normal 4-key recorder shortcuts fail.
