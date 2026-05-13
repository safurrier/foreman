---
id: macos-overlay-architecture
title: macOS Overlay Architecture
description: Architecture boundaries for Foreman's Swift macOS overlay prototype.
index:
  - id: boundaries
  - id: control-api-contract
  - id: swiftpm-modules
  - id: current-prototype-notes
  - id: known-hardening-work
---

# macOS Overlay Architecture

Foreman's macOS overlay is a native frontend over Foreman's Rust control plane.
The Swift app owns the macOS shell; Rust remains the source of truth for tmux and
harness interpretation.

```text
SwiftUI views → OverlayStore → ForemanClient → ProcessRunner → foreman binary → tmux
             ↘ OverlayAppRouting → OverlayPanelController / HotkeyController / TerminalActivator
```

## Boundaries

- **SwiftUI views** render state and send user intents. They do not spawn
  processes, know tmux syntax, or run AppleScript.
- **OverlayStore** owns query, normalized selection, loading/error state,
  compose draft, reload generation, and action dispatch. App effects leave the
  store through `OverlayAppRouting`.
- **ForemanClient** is a typed wrapper over the `foreman agents`, `foreman focus`, and `foreman send` commands.
- **ProcessRunner** is the only subprocess seam and must be fakeable. It uses an
  intentional `/usr/bin/env` path fallback for bare executables and typed launch
  errors for missing absolute paths.
- **OverlayPanelController** owns `NSPanel` creation, placement, show/hide, and
  focus behavior.
- **HotkeyController** owns Carbon global hotkey registration. The settings
  recorder persists the shortcut, but Foreman registers it so registration
  status reflects the real app-level hook.
- **Terminal activation** is isolated behind `TerminalActivating` adapters and
  not mixed into `ForemanClient`.

## Control API Contract

The Swift app consumes only stable command output:

```bash
foreman agents --json
foreman agents --json --pull-requests
foreman extensions --pane <PANE_ID> --json
foreman focus --pane <PANE_ID> --json
foreman send --pane <PANE_ID> --stdin --json
```

Rules:

- JSON goes to stdout.
- Human diagnostics/logging go to stderr.
- Pane ids are opaque strings.
- Preview text is bounded by the Rust control API.
- Schema changes require fixture updates and Swift decoder validation.
- The overlay loads agent/PR inventory first, then fetches extension cards for
  the selected pane so slow providers do not block first paint or hide PR cards.

## SwiftPM Modules

`ForemanOverlayCore` is split by ownership so reviews stay local:

- `Models.swift` — decoded control API models.
- `ForemanClient.swift` — typed CLI client.
- `ProcessRunner.swift` — subprocess adapter and timeout handling.
- `OverlayPreferences.swift` — persisted app preferences.
- `OverlayStore.swift` — observable state, deterministic reloads, and app
  routing seam.
- `KeyboardReducer.swift` — keyboard command/effect types.
- `RowPresenter.swift` — batched visible-row presentation.
- `TerminalActivationPreference.swift` — typed terminal activation preference.

The executable app shell remains under `apps/macos-overlay/Sources/ForemanOverlay/`; SwiftUI views
remain under `apps/macos-overlay/Sources/ForemanOverlayUI/`; deterministic rendering lives in
`apps/macos-overlay/Sources/ForemanOverlaySnapshot/`.

## Current Prototype Notes

- The app lives under `apps/macos-overlay/` as a SwiftPM package.
- The overlay uses `KeyboardShortcuts.Recorder` for persisted hotkey
  configuration and the same `KeyboardShortcuts` package for normal global
  shortcut handling. The default is `Cmd+Option+F`; `FOREMAN_OVERLAY_HOTKEY` can
  override it for development launches. Settings exposes separate **Clear
  Shortcut** and **Restore Default** actions so clearing the global shortcut is
  distinct from restoring the default chord.
- Launch with deterministic fake data using `scripts/capture-macos-overlay.sh`.
- Launch against the real local binary with:

```bash
FOREMAN_OVERLAY_FOREMAN_PATH=$PWD/target/debug/foreman \
FOREMAN_OVERLAY_SHOW_ON_LAUNCH=1 \
apps/macos-overlay/.build/debug/foreman-overlay
```

## Known Hardening Work

- Keep settings theme parity and multi-section settings snapshots on the polish backlog.
- Keep full PR actions (copy URL/refresh/status/detail parity) scoped separately from the quick tabbable `Open PR` card.
- Keep global hotkey behavior manually smoke-tested because system-wide shortcut
  conflicts and Secure Keyboard Entry are OS-level integration concerns.
