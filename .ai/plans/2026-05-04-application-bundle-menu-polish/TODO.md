---
id: macos-overlay-app-bundle-menu-polish-todo
title: TODO — macOS Overlay App Bundle and Menu Polish
---

# TODO

## Bundle / Spotlight / Raycast

- [x] Add `scripts/build-macos-overlay-app.sh`.
- [x] Generate `Foreman Overlay.app/Contents/Info.plist`.
- [x] Copy SwiftPM `foreman-overlay` executable into `Contents/MacOS/`.
- [ ] Add minimal app icon or placeholder.
- [x] Bundle local `foreman` binary under `Contents/Resources/foreman` for GUI launches.
- [x] Add `.mise/tasks/build-macos-overlay-app`.
- [x] Add `.mise/tasks/install-macos-overlay-app` to copy to `~/Applications`.
- [x] Document Spotlight/Raycast indexing steps.
- [x] Add validation that app bundle exists and has executable/plist.

## Menu Polish

- [x] Add standard top-left app menu when app is active:
  - About Foreman Overlay
  - Settings…
  - Hide Foreman Overlay
  - Quit Foreman Overlay
- [x] Add Foreman menu with:
  - Open Foreman Overlay
  - Refresh Agents
  - Settings…
- [x] Make app activation policy configurable:
  - accessory for quiet menu-bar mode
  - regular when launched as indexed app and settings/menu discoverability matters
- [x] Keep menu-bar item menu as fallback.

## Validation

- [x] Add bundle smoke to `verify-macos-overlay` and separate `verify-macos-overlay-app` task.
- [x] Launch bundled app in test mode and confirm it stays alive / writes panel snapshot.
- [ ] Confirm `open -a "Foreman Overlay"` works after install.

## Hyperkey note

- [ ] Do not deep-debug third-party hyperkey tooling unless normal recorder-set
      4-key shortcuts fail. Current evidence showed a persisted `Ctrl+B`
      shortcut, not a Carbon inability to represent `Ctrl+Option+Shift+A`.
