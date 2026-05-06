---
id: macos-foreman-ux-polish-investigation
title: Investigation — macOS Foreman UX Polish
---

# Investigation

User-reported issues from screenshots and manual use:

1. **List overlay visual bug**
   - Screenshot shows a blue `List` pill covering the selected row's right side.
   - Code root: `OverlayView.agentList` overlays `RegionBadge("List")` at
     `.topTrailing` on the whole `List`, so it floats above row content.
   - Product issue: active region chrome is debugging/status UI, but it is being
     rendered as content-obscuring chrome.

2. **Search placeholder copy**
   - Current placeholder: `Search Foreman agents`.
   - Desired: `Search Agent Sessions`.
   - Code root: hard-coded in `OverlayView.searchBar`.

3. **Settings didn't actually open**
   - `SettingsPanelController` creates a normal-level `NSPanel` while the main
     overlay panel is `.floating`.
   - Likely observed behavior: settings opens behind the floating overlay, which
     reads as no-op.
   - Fix direction: make settings panel floating/utility level, bring all windows
     forward, or hide/demote overlay before showing Settings.

4. **Help scroll also scrolls agent sessions**
   - Help is rendered as a ZStack overlay, but the underlying `List` and detail
     `ScrollView` remain live.
   - Pointer wheel/trackpad scroll can reach or affect underlying scroll views.
   - Fix direction: make Help a modal/event-intercepting overlay and/or disable
     underlying list/detail scrolling while help is visible.

5. **Cmd+T theme cycle causes system invalid-command sound**
   - Theme cycling is handled in a local key-down monitor.
   - AppKit still treats `Cmd+T` as an unhandled command/menu equivalent in some
     focus states and plays the invalid-command sound.
   - Fix direction: add a real `Cycle Theme` menu command with Cmd+T and route it
     to the same store action, or subclass the panel to handle key equivalents.

6. **Compose visual bug**
   - Screenshot shows the blue `Compose` pill overlaying the trailing edge of the
     compose text field/button area.
   - Code root: `RegionBadge("Compose")` is overlaid at `.topTrailing` on the
     `TextField`.
   - Same family as the List badge bug: overlay chrome is covering interactive
     content.

7. **Component styling/design-token weakness**
   - List/detail/compose/search use ad hoc modifiers and overlays.
   - Active-region badges are implemented as generic overlay pills rather than
     part of a layout system.
   - Fix direction: introduce small SwiftUI view primitives/tokens for search,
     rows, panels, footer chips, and focus indicators. Prefer layout-safe borders
     or footer/status chips over content-covering overlays.

8. **App naming**
   - User-facing app should be **Foreman**, not **Foreman Overlay**.
   - Rename bundle/menu/docs/scripts from `Foreman Overlay.app` to `Foreman.app`.
   - Keep internal executable/package names if useful (`foreman-overlay`) but all
     visible copy should say `Foreman`.

## Files implicated

- `apps/macos-overlay/Sources/ForemanOverlayUI/OverlayView.swift`
- `apps/macos-overlay/Sources/ForemanOverlay/main.swift`
- `scripts/build-macos-overlay-app.sh`
- `scripts/install-macos-overlay-app.sh`
- `scripts/verify-macos-overlay-app.sh`
- `scripts/verify-macos-overlay-snapshot-text.swift`
- `docs/macos-overlay/app-bundle.md`
- `.mise/tasks/*macos-overlay-app*`

## Screenshot evidence

- List badge overlap: `/var/folders/kf/js4h91w14pl7zwfgnvj896b00000gq/T/pi-clipboard-2e754f30-aa6c-46a7-b454-22dc902e295b.png`
- Compose badge overlap: `/var/folders/kf/js4h91w14pl7zwfgnvj896b00000gq/T/pi-clipboard-aca6d80c-474e-46c8-8fd5-87bb501b156f.png`
