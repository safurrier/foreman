---
id: macos-overlay-settings-filters-parity-spec
title: Spec — macOS Overlay Settings and Filters Parity
---

# Specification — macOS Overlay Settings and Filters Parity

## Problem

The macOS Foreman app is useful as a quick launcher/control surface, but the
settings and view controls are still prototype-level compared with the TUI:

- The app has no user-facing settings for default popup size.
- It does not expose enough Foreman settings/parity concepts such as terminal
  activation, notifications, all-panes visibility, sort/filter behavior, or
  harness/agent filters.
- The search bar has excess vertical space above the text/cursor and can make
  the field look visually misaligned.
- `Tab` cycles regions, but the detail/PR area does not get a clear/selectable
  detail/PR pane experience when a PR is present.
- The overlay lacks TUI-ish view controls like recent/attention sorting and
  specific harness/agent filters.

## Goals

- Add a Settings surface that feels like normal Foreman app preferences, not a
  single shortcut recorder.
- Add persistent popup size settings.
- Add persistent view settings for common Foreman overlay filters/sorts.
- Fix the search bar vertical alignment.
- Make `Tab`/region focus visibly and functionally select the detail/PR area.
- Keep the overlay command-palette default simple.

## Non-goals

- Full TUI settings parity in one pass.
- Notification delivery implementation if the runtime notification system is not
  already exposed through the control API.
- A large SwiftUI design system rewrite.

## Requirements

### Settings

Settings MUST include sections/tabs for:

1. **General**
   - Global shortcut recorder.
   - Reset shortcut button.
   - Default popup width/height controls.
   - Reset popup size.
2. **View**
   - Include non-agent panes / all panes.
   - Sort mode: stable, attention first, recent activity first.
   - Harness filter: all, Pi, Claude Code, Codex CLI, Gemini, OpenCode, unknown.
   - Optional status filter: all, working, needs attention, idle, error.
3. **Focus**
   - Terminal activation: auto, none, Ghostty, iTerm2, Terminal, WezTerm,
     Alacritty, Kitty, custom bundle id.
4. **Notifications** placeholder/parity section
   - Show current status as not implemented or requires future control API if
     needed.
   - Do not imply unsupported notification delivery works.

Settings SHOULD persist via UserDefaults under the active app domain
`dev.foreman.app`.

### Popup size

- Panel size MUST use persisted width/height on launch.
- Width/height MUST have sensible min/max bounds.
- User SHOULD be able to resize the panel directly if feasible.
- Persisted size SHOULD update after resize or when Settings values change.

Initial suggested bounds:

- width: 720–1400, default 820
- height: 480–1000, default 560

### Search bar visual fix

- Search bar height/padding MUST be tightened so the text cursor and placeholder
  align visually near the center of the search row.
- The bar SHOULD not reserve large blank vertical space above the field.
- Validate via headless/live screenshot.

### Detail / PR region

- `Tab` to Details MUST make the detail area's active region obvious.
- If a PR card is present, the active detail region SHOULD expose keyboard
  actions in the footer/help:
  - open PR
  - copy PR URL if feasible
- If full keyboard interaction with SwiftUI `Link` is awkward, use a Button that
  opens the URL via `NSWorkspace` behind a store callback.

### View filters / sorting

OverlayStore SHOULD support:

- `includeAllPanes` / all panes toggle, wired to `foreman agents --json --all-panes`.
- sort mode:
  - stable/current API order
  - attention first
  - recent activity first
- harness filter:
  - all
  - specific harness label/type
- status filter:
  - all
  - working
  - needs attention
  - idle
  - error

The footer or search area SHOULD display active filters compactly when not
default.

### Validation

- Swift unit tests for filtering/sorting and settings persistence seam.
- Snapshot states for:
  - default size
  - large size if snapshot renderer supports dimensions
  - active filters
  - details active with PR card
  - settings view if practical
- `mise run verify-macos-overlay` passes.

## Acceptance

- User can set popup width/height in Settings and see it apply on next open.
- Search bar cursor no longer appears vertically awkward with large blank space.
- Tab to Details/PR has an obvious active detail region and footer help.
- View filters/sort can be changed and persist.
- Settings documents notification parity honestly.
