---
id: plan-implementation
title: Implementation Plan
description: >
  Slice plan for turning the Swift macOS overlay from a working prototype into a
  validated daily-driver control surface. Packaging/notarization is explicitly
  out of scope for this plan.
---

# Implementation — macos-overlay-prototype

## Direction

Treat the Swift overlay as a **Mac-native command palette for Foreman**, not a
second implementation of the Ratatui app. Parity means the same core operator
workflows work reliably from the overlay:

```text
open → search/navigate → inspect → focus/send → close
```

The overlay should borrow Foreman's validation style: fast pure tests first,
fake harnesses for external seams, visual artifacts for UI states, then focused
real-tmux smoke tests.

Packaging, notarization, launch-at-login, and distribution are out of scope for
this plan.

## Slice 1 — Interaction parity foundation

Goal: make the overlay feel like a keyboard-first Foreman/Raycast-style control
surface.

### Product work

- Keep normal typing routed to search whenever compose is not active.
- Keep `↑` / `↓` as agent navigation.
- Make list scrolling follow keyboard selection when moving beyond visible rows.
- Add focus regions similar to Foreman:
  - list/sidebar
  - detail/preview
  - compose/input
- Add `Tab` / `Shift+Tab` region cycling.
- Add `Enter` as focus selected agent from list/search.
- Add `Cmd+R` refresh.
- Add `Esc` close/cancel behavior:
  - close help first
  - cancel compose first
  - otherwise close overlay
- Add clear hints in the footer for the current region/mode.

### Validation

- Swift unit tests for command reducer/state transitions once keyboard behavior
  is extracted from `main.swift`.
- Manual smoke against real Foreman overlay.
- Update screenshot fixture after visual changes.

### Done when

- User can open the overlay and operate the happy path without touching the
  mouse.
- Arrow selection never disappears off-screen without the list scrolling.
- Region focus is visible and predictable.

## Slice 2 — Help surface and keyboard-scrollable details

Goal: close the obvious gap versus Foreman's `?` help and scrollable panels.

### Product work

- Add `?` help overlay/sheet.
- Include:
  - global shortcuts
  - list/search shortcuts
  - detail/preview shortcuts
  - compose shortcuts
  - status/source legend
  - explanation of real vs compatibility/native status provenance
- Make help keyboard-scrollable with `j/k`, arrow keys, PageUp/PageDown, or
  standard scroll behavior.
- Make preview/detail pane keyboard-scrollable when detail region is active.
- Preserve text selection in terminal preview for mouse users.

### Validation

- Unit tests for `?`, help scroll, and `Esc` close behavior in the overlay state
  model.
- Add visual fixture/capture for help at normal size and constrained height.
- Add gauntlet assertion that help content below the fold can be reached by
  keyboard.

### Done when

- `?` opens useful help.
- Keyboard scrolling can reveal help content hidden below the fold.
- `Esc` returns to the previous overlay mode without losing search/selection.

## Slice 3 — App-level fake-Foreman action gauntlet

Goal: prove actual UI actions call the expected Foreman control API, not just
that the core client works in isolation.

### Product/test work

- Add an app-level test harness that launches the overlay against a fake
  Foreman executable.
- Record fake Foreman calls and stdin.
- Drive enough UI events to prove:
  - launch calls `agents --json`
  - double-click row calls `focus --pane <id> --json`
  - `Enter` calls `focus --pane <id> --json`
  - compose + `Cmd+Enter` calls `send --pane <id> --stdin --json`
  - refresh calls `agents --json` again
- Prefer XCUITest if stable. If XCUITest is too brittle for a menu-bar accessory
  app, add a first-class app test mode that accepts scripted keyboard/click
  events and exits after writing assertions.

### Validation

- New local command, likely:

```bash
mise run macos-overlay-gauntlet
```

- The command writes artifacts under:

```text
.ai/validation/macos-overlay/gauntlet/
├── fake-foreman-calls.log
├── fake-foreman-stdin.log
├── overlay.log
└── summary.md
```

### Done when

- The gauntlet fails if focus/send/refresh are not wired through the UI.
- The gauntlet is deterministic and does not require live tmux.

## Slice 4 — Visual state fixture gauntlet

Goal: make UI iteration deterministic across the important Foreman states.

### Product/test work

Add fixtures under `apps/macos-overlay/Fixtures/`:

- `agents-attention.json`
- `agents-empty.json`
- `agents-error.json` or a fake command failure mode
- `agents-long-path.json`
- `agents-long-preview.json`
- `agents-many.json`
- `agents-mixed-status.json`

Extend `scripts/capture-macos-overlay.sh` so it can capture one named state or
all states.

Expected artifacts:

```text
.ai/validation/macos-overlay/screenshots/
├── attention.png
├── empty.png
├── error.png
├── long-path.png
├── long-preview.png
├── many.png
└── mixed-status.png
```

### Validation

- Assert each requested screenshot file exists and is non-empty.
- Assert fake Foreman logs include expected calls.
- Human review screenshots for:
  - selected row visibility
  - detail hierarchy
  - footer hints
  - truncation/wrapping
  - status/source clarity
  - error recovery text

### Done when

- `mise run capture-macos-overlay` can regenerate all key visual states.
- New UI changes can be reviewed from artifacts without live tmux state.

## Slice 5 — Foreman feature-parity matrix

Goal: explicitly decide which Foreman workflows the overlay must match, which it
adapts, and which it defers.

The overlay should not become a full TUI clone. It should cover the high-value
operator paths that benefit from global access.

### Initial parity targets

| Foreman capability | Overlay target | Notes |
|---|---|---|
| Agent discovery | Must | `agents --json`, default agent list |
| Search | Must | type-to-search, no explicit `/` required |
| Keyboard movement | Must | arrows now; consider `j/k` too |
| Selection scroll visibility | Must | list must follow selected row |
| Focus selected pane | Must | Enter, double-click, button |
| Direct input/send | Must | compose mode + send |
| Help `?` | Must | overlay-specific help, scrollable |
| Focus regions | Must | list/detail/compose via Tab/Shift+Tab |
| Preview scrolling | Should | keyboard scroll in detail region |
| Status/source provenance | Should | visible native/compat/source label |
| Non-agent panes / all panes | Should | expose `--all-panes` through a toggle if useful |
| Harness/provider filter | Should | likely useful once list grows |
| Theme cycling | Could defer | Mac system appearance may be enough |
| PR panel | Defer | requires control API expansion beyond current agents/focus/send |
| Spawn/rename/kill windows | Defer | higher-risk operations; not needed for global quick overlay |
| Notifications | Defer | main Foreman already owns notification behavior |

### Validation

- Add this matrix to `docs/macos-overlay/validation.md` or a nearby doc and keep
  it updated.
- Each Must/Should parity item gets at least one proof layer:
  - unit test
  - fake app gauntlet
  - screenshot
  - real tmux smoke
  - manual Mac smoke

### Done when

- Gaps are intentional rather than accidental.
- The overlay has a clear contract distinct from the TUI.

## Slice 6 — Configurable hotkey and menu-bar polish

Goal: make the overlay a better Mac citizen without packaging it yet.

### Product work

- Replace hardcoded `Ctrl+Option+Cmd+F` with `KeyboardShortcuts` or equivalent.
- Keep the current chord as the default.
- Add a small Settings surface for the hotkey.
- Improve menu-bar label/icon.
- Add menu items with stable names and shortcuts:
  - Open Foreman Overlay
  - Refresh Agents
  - Settings
  - Quit

### Validation

- Swift unit tests where possible for settings persistence/defaults.
- Manual Mac smoke for hotkey registration and conflict handling.

### Done when

- Hotkey can be changed without code edits.
- Menu-bar item communicates Foreman rather than generic `⌘F`.

## Slice 7 — Terminal activation adapter

Goal: decide what happens after `focus` succeeds and isolate OS-specific logic.

### Product work

- Add a small adapter boundary, e.g. `TerminalActivating`.
- Supported initial modes:
  - none: Foreman focuses tmux only, overlay closes
  - activate frontmost/configured terminal app after focus
- Keep AppleScript/AppKit activation details out of SwiftUI views.
- Make failures non-fatal and visible in logs/status.

### Validation

- Unit test adapter selection/config behavior.
- Manual Mac smoke for Terminal/iTerm/WezTerm/Ghostty as available.
- Real tmux smoke still proves Foreman focus independently of app activation.

### Done when

- Focus semantics are explicit.
- Terminal activation can be enabled/disabled without touching UI code.

## Cross-slice validation command target

Long term local command:

```bash
mise run verify-macos-overlay
```

Should cover:

1. `cargo test control_api --lib`
2. `swift test --package-path apps/macos-overlay`
3. app-level fake-Foreman action gauntlet
4. visual fixture capture/assertions
5. real tmux control API smoke

Manual-only checks remain documented separately:

- global hotkey registration
- Spaces/fullscreen behavior
- terminal activation behavior
- visual feel in light/dark appearance
