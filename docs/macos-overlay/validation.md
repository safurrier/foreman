---
id: macos-overlay-validation
title: macOS Overlay Validation
description: Deterministic validation ladder for the Swift macOS Foreman overlay.
index:
  - id: validation-ladder
  - id: headless-visual-snapshots
  - id: live-visual-capture
  - id: required-overlay-change-lane
  - id: manual-terminal-activation-smoke
  - id: foreman-style-overlay-gauntlet
  - id: current-gaps
---

# macOS Overlay Validation

The macOS overlay should follow the same validation philosophy as the Rust TUI:
prove pure state with fast tests, cross external seams through fakes first, then
keep one focused real-environment smoke for tmux/Foreman behavior.

## Validation Ladder

| Layer | Proof |
|---|---|
| Swift build | `swift build --package-path apps/macos-overlay` |
| Rust control API | `cargo test control_api --lib` and control-command tests |
| Client contract | Decode checked-in `foreman agents --json` fixtures with Swift models |
| Fake process seam | Fake `foreman` executable records `agents`, `focus`, and `send` calls |
| Preferences/filter behavior | Swift unit tests for UserDefaults-backed popup size, terminal activation settings, sort, harness filter, status filter, and all-panes argument wiring |
| Headless visual artifact | Render SwiftUI snapshots without showing the panel to `.ai/validation/macos-overlay/headless-snapshots/` |
| Live visual smoke | Optionally launch overlay against fake `foreman`, capture screenshot to `.ai/validation/macos-overlay/screenshots/` |
| Real tmux smoke | Temporary tmux server plus `target/debug/foreman` agents, focus, and send commands |
| Manual Mac UX | Global hotkey, focused display, Spaces/full-screen, terminal activation |

## Headless Visual Snapshots

Use the non-disruptive render path for routine visual validation:

```bash
mise run render-macos-overlay-snapshots
```

It builds `foreman-overlay-snapshot`, renders `OverlayView` through an offscreen
`NSHostingView`, and writes PNGs without showing the floating panel or stealing
focus.

Expected snapshot state names are owned by the snapshot renderer. To inspect the current registry:

```bash
swift build --package-path apps/macos-overlay
apps/macos-overlay/.build/debug/foreman-overlay-snapshot --list-states
```

`scripts/render-macos-overlay-snapshots.sh` and `scripts/verify-macos-overlay.sh` consume that registry so adding a new snapshot state only requires updating the renderer plus OCR expectations when semantic text checks are needed.

## Live Visual Capture

Use live panel capture only when validating AppKit panel behavior:

```bash
./scripts/capture-macos-overlay.sh
```

It:

1. Builds the Swift overlay.
2. Creates a temporary fake `foreman` executable.
3. Launches the overlay with `FOREMAN_OVERLAY_SHOW_ON_LAUNCH=1`.
4. Points the overlay at a deterministic fixture under `apps/macos-overlay/Fixtures/`.
5. Captures a screenshot using `screencapture`.
6. Writes evidence under `.ai/validation/macos-overlay/`.

Live capture ergonomics:

- The script launches with `FOREMAN_OVERLAY_ACTIVATE=0` by default so screenshot
  capture is less likely to steal keyboard focus while you work.
- The script launches with `FOREMAN_OVERLAY_JOIN_ALL_SPACES=0` by default so the
  validation panel does not intentionally follow every Space.
- Set `FOREMAN_OVERLAY_SCREEN_INDEX=<n>` to place the overlay on a specific
  physical display when multiple monitors are attached.
- macOS Spaces/Desktop targeting is not exposed through a stable public API. To
  capture on Desktop 2, switch to that Space before running the capture, or run
  an explicit local automation that switches Spaces first. Do not rely on Spaces
  automation in CI.

## Fixture Rules

- Fixtures should be small, deterministic, and checked in under
  `apps/macos-overlay/Fixtures/`.
- Cover at least: empty, attention, working, long-path/long-preview, and error
  states before claiming visual regression confidence.
- Do not use live tmux inventory for visual iteration. Live state belongs in the
  real smoke layer, not screenshot review.

## Required Overlay Change Lane

For changes under `apps/macos-overlay/`, `scripts/macos-overlay-*`, the control
API used by the overlay, or keyboard/focus/settings behavior, run the required
local lane before committing or opening a PR:

```bash
mise run validate-macos-overlay-change
```

This is an alias for the full non-release overlay verifier. It runs Rust control
API tests, Swift unit tests, the fake-Foreman UI event gauntlet, real tmux smoke,
headless snapshots with OCR semantic checks, and app bundle smoke. Use plain
`swift test --package-path apps/macos-overlay` only for inner-loop TDD.

## CI / Local Split

- Fake process, fixture decoding, preferences, filter/sort, reload, and process
  runner tests are Swift unit tests.
- The fake-Foreman UI event gauntlet is required in the overlay validation lane,
  even though it remains skipped in plain `swift test` unless
  `FOREMAN_OVERLAY_RUN_UI_TESTS=1` is set.
- Screenshot capture is macOS-only and may be local-only until CI has a stable
  window server.
- Global hotkey behavior should remain manual/local. Tests should use
  `FOREMAN_OVERLAY_SHOW_ON_LAUNCH=1` instead of trying to synthesize the hotkey.
- The normal persisted hotkey UI uses `KeyboardShortcuts.Recorder` in the menu
  bar Settings panel, while Foreman registers the chosen shortcut through its own
  lifecycle-safe Carbon controller.
- The default persisted hotkey is `Ctrl+F`.
- `FOREMAN_OVERLAY_HOTKEY` can override the persisted shortcut for local testing,
  using strings like `control+f` or `cmd+shift+space`.
- `FOREMAN_OVERLAY_TERMINAL_ACTIVATION` controls terminal foregrounding after
  focus. The default is `auto`, which returns to the most recently active known
  terminal or the first running known terminal. Supported values: `auto`, `none`,
  `terminal`, `iterm`, `ghostty`, `wezterm`, `alacritty`, `kitty`, or
  `bundle:<bundle-id>`.

## Manual Terminal Activation Smoke

Terminal foregrounding is intentionally local/manual because it depends on which
terminal apps are installed and running.

Default behavior uses automatic terminal activation:

```bash
FOREMAN_OVERLAY_TERMINAL_ACTIVATION=auto \
FOREMAN_OVERLAY_FOREMAN_PATH=$PWD/target/debug/foreman \
FOREMAN_OVERLAY_SHOW_ON_LAUNCH=1 \
apps/macos-overlay/.build/debug/foreman-overlay
```

Disable foregrounding or pin a specific terminal with:

```bash
FOREMAN_OVERLAY_TERMINAL_ACTIVATION=none      # no app activation after focus
FOREMAN_OVERLAY_TERMINAL_ACTIVATION=terminal  # Terminal.app
FOREMAN_OVERLAY_TERMINAL_ACTIVATION=iterm     # iTerm2
FOREMAN_OVERLAY_TERMINAL_ACTIVATION=ghostty   # Ghostty
FOREMAN_OVERLAY_TERMINAL_ACTIVATION=wezterm   # WezTerm
FOREMAN_OVERLAY_TERMINAL_ACTIVATION=alacritty # Alacritty
FOREMAN_OVERLAY_TERMINAL_ACTIVATION=kitty     # Kitty
FOREMAN_OVERLAY_TERMINAL_ACTIVATION=bundle:com.example.Terminal
```

Manual proof checklist:

1. Start Foreman overlay against real `target/debug/foreman`.
2. Select an agent whose pane is visible in the configured terminal.
3. Press Enter or double-click the row.
4. Confirm the overlay closes.
5. Confirm tmux focus moved to the pane.
6. If activation is enabled, confirm the configured terminal app comes forward.
7. If the configured app is not running, confirm focus still succeeds and the
   overlay logs a non-fatal activation skip.

## Foreman-Style Overlay Gauntlet

Foreman's Rust TUI has release gauntlets that prove full workflows. The overlay
should grow a smaller equivalent that proves command-palette parity rather than
full TUI cloning.

The gauntlet should cover:

1. launch against fake Foreman and load `agents --json`
2. type-to-search
3. keyboard navigation with selected-row visibility
4. focus selected pane via Enter and double-click
5. compose/send via `Cmd+Enter`
6. `?` help visibility and keyboard scrolling
7. `Tab` / `Shift+Tab` movement between list, detail, and compose regions
8. headless visual captures for important UI states
9. real tmux smoke for the underlying agents, focus, and send control API

Feature parity should be explicit:

| Foreman capability | Overlay target |
|---|---|
| Agent discovery | Must |
| Search | Must, type-to-search by default |
| Keyboard movement | Must |
| Selected-row scroll visibility | Must |
| Focus selected pane | Must |
| Direct input/send | Must |
| Help `?` | Must, keyboard-scrollable |
| Focus regions | Must, list/detail/compose |
| Preview scrolling | Should |
| Status/source provenance | Should |
| Non-agent panes / all panes | Should decide |
| Harness/provider filter | Should decide |
| Theme cycling | Added lightweight `Cmd+T` overlay themes |
| PR panel | Partial: clickable PR link via `--pull-requests`; full TUI PR actions deferred |
| Spawn/rename/kill | Defer |
| Notifications | Defer to Foreman runtime |

Detailed working plan lives in
`.ai/plans/2026-05-03-171708-macos-overlay-prototype/GAUNTLET.md`.

## Current Gaps

- Screenshot capture is full-screen rather than window-scoped.
- Global hotkey behavior remains manual/local because system-wide shortcuts are
  OS-level integration, but Settings now surfaces Carbon registration health.
- More Settings section snapshots are still needed beyond the General pane.
- A future CI lane should run `mise run validate-macos-overlay-change` on a
  hosted macOS runner with a stable window server.
