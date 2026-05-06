---
id: macos-foreman-ux-polish-spec
title: Spec — macOS Foreman UX Polish
---

# Specification — macOS Foreman UX Polish

## Problem

The native macOS app is now launchable, but several UI details make it feel like
a prototype instead of a polished Mac control surface:

- visible active-region badges overlap content
- Settings appears not to open
- Help scrolling leaks to the background list
- Cmd+T theme cycling can play the invalid-command sound
- search copy is too implementation-oriented
- user-facing app name says `Foreman Overlay` instead of `Foreman`

## Product direction

The app should feel like **Foreman for macOS**: a native command-palette control
surface for agent sessions. The internal term "overlay" is useful for code and
docs, but not for app chrome.

## Requirements

### Naming

- User-facing app name MUST be `Foreman`.
- Bundle install path SHOULD be `~/Applications/Foreman.app`.
- `open -a Foreman` MUST launch the app.
- Existing internal executable/package names MAY remain `foreman-overlay`.
- Docs and validation should explain the transition from the earlier
  `Foreman Overlay.app` prototype bundle.

### Search

- Search placeholder MUST say `Search Agent Sessions`.
- Snapshot OCR expectations MUST be updated.

### Settings

- Settings MUST visibly appear above the overlay when opened from:
  - footer button
  - app menu
  - status menu
  - Cmd+,
- Settings SHOULD use an appropriate floating/utility panel level or hide/demote
  the overlay first.
- Settings SHOULD be part of the app menu under standard `Settings…`.

### Active-region UI

- List/detail/compose active-region indicators MUST NOT cover content.
- Replace floating blue `RegionBadge` overlays with layout-safe focus treatment:
  - subtle border/ring around active panel, and/or
  - footer chips/hints, and/or
  - inline header state.
- Compose field MUST keep the full text input and send button/affordance clear.

### Help modal

- Help MUST behave modally while visible.
- Pointer/trackpad scroll over help MUST NOT scroll the underlying agent list or
  detail preview.
- Keyboard help scrolling MUST remain supported.

### Theme hotkey

- Cmd+T MUST cycle themes without system beep.
- Add a real menu command or AppKit key-equivalent handler rather than relying
  only on the local key-down monitor.

### Component/style cleanup

- Introduce small SwiftUI styling primitives for the overlay, enough to prevent
  repeated badge-over-content bugs:
  - focus ring/panel style
  - footer command chip or hint style
  - compose input row style
  - search field title/copy constants
- Avoid a large design-system rewrite.

## Acceptance

- `mise run render-macos-overlay-snapshots` produces screenshots with no badge
  overlap in list/detail/compose/help states.
- OCR checks pass with `Search Agent Sessions` and `Foreman Help`/`Foreman` copy.
- Manual launch from `open -a Foreman` opens the panel.
- Settings opens visibly above the overlay.
- Cmd+T cycles theme silently.
- Pointer scrolling the Help overlay does not move the underlying list.
- `mise run verify-macos-overlay` passes.
