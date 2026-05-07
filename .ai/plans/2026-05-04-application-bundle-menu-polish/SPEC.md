---
id: macos-overlay-app-bundle-menu-polish-spec
title: macOS Overlay App Bundle and Menu Polish
---

# Specification — macOS Overlay App Bundle and Menu Polish

## Problem

The Swift overlay currently runs as a SwiftPM executable. It can be launched from
shell scripts, but macOS does not index it as a normal application for Spotlight,
Raycast, Dock/Finder, or a standard top-left app menu experience.

The app also runs as an accessory/menu-bar utility, so the menu bar does not show
an obvious **Foreman** application menu while other applications are active.
Users need a discoverable way to open Settings and the overlay without hunting
for the menu-bar item.

## Goals

- Produce a local `.app` bundle for the overlay.
- Make the bundle indexable by Spotlight/Raycast when installed in
  `/Applications` or `~/Applications`.
- Add a standard app menu surface with stable command names.
- Keep menu-bar utility behavior for daily use.
- Keep validation deterministic and local-friendly.

## Non-goals

- Notarization/signing/distribution.
- Launch-at-login.
- Homebrew cask / installer.
- Debugging third-party hyperkey tools unless the app fails to record a normal
  4-key shortcut through `KeyboardShortcuts`.

## Requirements

### MUST

- Add an app bundling script that creates `Foreman Overlay.app` from the SwiftPM
  debug executable.
- Generate a valid `Info.plist` with stable bundle id, executable name, display
  name, minimum macOS version, and agent/accessory behavior.
- Add a mise task to build/install the local app bundle.
- Document how to install to `~/Applications` and refresh Spotlight/Raycast.
- Add app/menu commands:
  - About Foreman Overlay
  - Open Foreman Overlay
  - Refresh Agents
  - Settings…
  - Quit Foreman Overlay
- Keep the menu-bar item as a fallback command surface.

### SHOULD

- Add a small app icon or placeholder icon.
- Add a visible Settings button in the overlay footer.
- Preserve `FOREMAN_OVERLAY_FOREMAN_PATH` support for development launches.
- Add smoke validation that the `.app` bundle exists, contains executable and
  `Info.plist`, and can launch/exit in a test mode.

## Acceptance

- `mise run build-macos-overlay-app` creates `apps/macos-overlay/dist/Foreman Overlay.app`.
- `mise run install-macos-overlay-app` copies it to `~/Applications/Foreman Overlay.app`.
- `mdls ~/Applications/Foreman\ Overlay.app` can see the app bundle after
  Spotlight indexes it.
- App launch command works:

```bash
open -a "Foreman Overlay"
```

- `mise run verify-macos-overlay` still passes.
