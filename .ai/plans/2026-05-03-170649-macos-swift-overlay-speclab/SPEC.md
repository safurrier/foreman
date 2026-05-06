---
id: macos-swift-overlay-speclab-spec
title: Swift macOS Overlay Speclab Specification
description: >
  Research-backed product and engineering specification for a simple native
  macOS Foreman overlay, plus validation and skill-building work to do before
  implementation.
---

# Specification — macos-swift-overlay-speclab

## Problem

The Raycast extension path gives Foreman a quick list/detail surface, but it
still lives inside Raycast’s launcher model. The desired product is closer to a
Foreman-owned control-plane HUD: press one global hotkey from anywhere on macOS,
see live-ish agent status, search/select a pane, preview recent output, focus or
send input, then dismiss.

Before implementing, we need a speclab pass that answers:

- What similar macOS command-palette/launcher apps teach us about UX.
- What simple Swift/AppKit/SwiftUI architecture fits this repo.
- How to validate the Swift “agent harness” boundary with the same discipline as
  Foreman’s Rust/tmux/harness validation ladder.
- What repo-local skills/guides should exist before agents start coding the app.

## Product Direction

Build a **small native macOS companion app** that uses Foreman’s future JSON and
control CLI as its backend.

```text
global hotkey → Foreman overlay → agents list/detail/compose → foreman CLI → tmux
```

Foreman Rust remains the truth/control plane. Swift owns the native macOS shell,
windowing, hotkey, visual presentation, and app lifecycle.

## Similar App UX Findings

### Spotlight

- Opens with a global hotkey and focuses search immediately.
- Results update as the user types.
- Keyboard is primary: arrows navigate, Return opens, Space previews, Command can
  reveal location/details.
- The panel is transient, spatially stable, and draggable/resizable.

Sources:

- https://support.apple.com/guide/mac-help/search-with-spotlight-mchlp1008/mac
- https://support.apple.com/en-il/guide/mac-help/mh26783/mac

### Raycast

- Uses a root search bar as central hub, with commands/extensions/files as
  results.
- Supports configurable hotkeys, aliases, compact/expanded modes, window
  position, focused-display behavior, and auto-close on focus loss.
- Action panels keep secondary actions discoverable without overloading rows.

Sources:

- https://manual.raycast.com/search-bar
- https://manual.raycast.com/settings

### Alfred

- Optimizes for launcher breadth: clipboard, snippets, calculator, workflows,
  system commands, terminal/shell commands, previews, and appearance settings.
- The lesson for Foreman is not to copy breadth. Copy the speed, keyboard-first
  behavior, row density, and workflow/action extensibility.

Source:

- https://www.alfredapp.com/help/overview/

### Command palette pattern

- Good anatomy: trigger, search input, grouped results, row label/metadata/icon,
  empty/loading/error states, and explicit keyboard behavior.
- Major risks are state management, accessibility, async recovery, and layout
  stability more than the default visual state.

Source:

- https://uxpatterns.dev/patterns/advanced/command-palette

## UX Requirements

### P0 — Native overlay MVP

- A single global hotkey toggles the Foreman overlay from anywhere on macOS.
- The search field is focused on open.
- Overlay appears on the focused display, centered or top-centered, with stable
  dimensions.
- Agent rows show status, harness, pane/session/window label, workspace, and age
  or source if available.
- The selected row shows a preview/details pane with recent output, status source,
  pane id, cwd, and useful next actions.
- Keyboard operations:
  - `Esc`: close overlay.
  - `↑/↓` or `Ctrl+P/N`: move selection.
  - `Enter`: focus selected tmux pane.
  - `Tab` or `Cmd+Enter`: compose/send to selected pane.
  - `Cmd+R`: refresh.
- Empty/error/loading states are explicit and actionable.
- The app never requires the user to open a terminal to inspect ordinary errors;
  it should point to `foreman --doctor` when backend setup looks broken.

### P1 — Daily-driver polish

- Configurable global hotkey.
- Menu bar item for open/settings/quit/doctor.
- Fuzzy filtering over title, session, harness, cwd, status, and preview snippets.
- Attention-first default ordering, with stable behavior across refreshes.
- Optional auto-refresh while open, with stale indicator.
- Terminal activation after successful focus if tmux selection alone does not
  foreground the terminal app.

### P2 — Later polish

- Compact/expanded layout preference.
- Visual themes aligned with Foreman’s terminal palettes.
- Recent/favorite panes.
- Native notifications or menu bar status badge.
- Packaged release path if the companion becomes more than a personal tool.

## Technical Architecture

### Recommended stack

- SwiftUI for overlay content and settings views.
- AppKit `NSPanel` for the floating/nonactivating overlay window.
- `NSHostingController` or `NSHostingView` to host SwiftUI inside the panel.
- `KeyboardShortcuts` package for user-configurable global hotkeys.
- `Foundation.Process` behind a protocol for invoking `foreman`.
- XCTest/Swift Testing for model/client tests; XCUITest/manual smoke for true
  macOS window/hotkey behavior.

### Why AppKit + SwiftUI

SwiftUI is good for the list/detail/settings UI, but AppKit is still the right
place to own macOS windowing details: floating panels, nonactivating behavior,
focus/key-window rules, collection behavior across Spaces/full-screen apps, and
menu bar lifecycle.

### Core components

| Component | Responsibility |
|---|---|
| `ForemanOverlayApp` | Swift app entry point, menu bar/settings scenes, dependency wiring |
| `OverlayPanelController` | Create/retain/show/hide `NSPanel`, position on focused screen, manage key focus |
| `HotkeyController` | Register configurable global shortcut and call overlay toggle |
| `OverlayStore` | Single observable state source: query, rows, selection, loading/error, compose draft |
| `ForemanClient` | Typed async API: `agents()`, `focus(paneID)`, `send(paneID,text)` |
| `ProcessRunner` | Low-level process execution with stdout/stderr/status/timeout/cancel |
| `AgentFilter` | Pure fuzzy filtering/sorting over decoded agent rows |
| `TerminalActivator` | Optional macOS app activation after focus; isolated from Foreman client |

### Dependency direction

```text
SwiftUI views → OverlayStore → ForemanClient → ProcessRunner → foreman binary
             ↘ OverlayPanelController / HotkeyController at app boundary only
```

Views should never construct `Process`, call AppleScript, or know tmux syntax.
The Swift app treats Foreman JSON/control commands as its backend API.

## External Research Findings

- `KeyboardShortcuts` provides user-customizable global shortcuts, a SwiftUI
  recorder, UserDefaults persistence, conflict warnings, key-up/key-down APIs,
  and is sandbox/App Store compatible. It explicitly says to prefer customizable
  shortcuts and avoid stealing user shortcuts. Source:
  https://github.com/sindresorhus/KeyboardShortcuts
- `HotKey` is simpler for hardcoded shortcuts and wraps Carbon global hotkey
  APIs. Its registration follows object lifecycle, so retaining the `HotKey`
  instance matters. Source: https://github.com/soffes/HotKey
- Apple’s `NSPanel` is the AppKit primitive for auxiliary/floating windows; use
  it for a command palette overlay rather than trying to make a normal SwiftUI
  window behave like Spotlight. Source: https://developer.apple.com/documentation/appkit/nspanel
- Apple’s `MenuBarExtra` supports persistent menu bar utilities in SwiftUI, but
  it is macOS 13+. Source: https://developer.apple.com/documentation/SwiftUI/MenuBarExtra
- Apple’s SwiftUI model guidance recommends keeping models separate from views
  and using observable state so views read only what they need. Source:
  https://developer.apple.com/documentation/SwiftUI/Managing-model-data-in-your-app
- Apple’s `NSEvent.addGlobalMonitorForEvents` can observe global events but
  cannot modify/prevent them, and key events require Accessibility trust. Prefer
  a hotkey registration library for the primary shortcut. Source:
  https://developer.apple.com/documentation/appkit/nsevent/addglobalmonitorforevents%28matching%3Ahandler%3A%29
- Apple’s `Process` is the subprocess primitive, but sandboxed children inherit
  the parent sandbox. Wrap it in a protocol and keep CLI interaction explicit.
  Source: https://developer.apple.com/documentation/Foundation/Process
- `SnapshotTesting` can snapshot images, recursive descriptions, JSON, dumps,
  and other diffable values across Swift platforms. Use it for visual/data
  regression where stable. Source: https://github.com/pointfreeco/swift-snapshot-testing
- `ViewInspector` can introspect SwiftUI views in unit tests and trigger control
  callbacks. Use it sparingly for focused view logic, not as the only UX proof.
  Source: https://github.com/nalexn/ViewInspector

## Validation Strategy

The Swift overlay should get its own validation ladder, analogous to Foreman’s
Rust ladder in `docs/workflows.md`.

| Layer | Proof |
|---|---|
| Model/unit | `OverlayStore`, filtering, selection, command mapping, error state |
| Client contract | Decode Foreman JSON fixtures; fake `ProcessRunner`; verify args/stdin/status handling |
| View unit | SwiftUI row/detail/empty/error states with ViewInspector or plain store-driven tests |
| Snapshot | Stable overlay states: empty, loading, attention rows, error, compose |
| AppKit harness | `OverlayPanelController` show/hide/focus positioning under test-friendly hooks |
| XCUITest | Launch app in deterministic mode with fake `foreman`; navigate keyboard-only; verify fake CLI log |
| Real tmux smoke | Launch overlay against real `foreman` and temporary tmux fixture; focus/send selected pane |
| Manual UX checklist | Global hotkey from another app, focused display, terminal activation, Spaces/full-screen behavior |

### Agent harness-style validation for Swift

Create a fake Foreman executable for tests:

```sh
#!/bin/sh
printf '%s\n' "$*" >> "$FOREMAN_OVERLAY_CALL_LOG"
case "$1" in
  agents) cat "$FOREMAN_OVERLAY_FIXTURE_JSON" ;;
  focus)  printf '{"ok":true}\n' ;;
  send)   cat > "$FOREMAN_OVERLAY_STDIN_LOG"; printf '{"ok":true}\n' ;;
  *)      echo "unknown command" >&2; exit 64 ;;
esac
```

Then run app/UI tests with launch environment:

```text
FOREMAN_OVERLAY_FOREMAN_PATH=/tmp/fake-foreman
FOREMAN_OVERLAY_FIXTURE_JSON=Tests/Fixtures/agents-attention.json
FOREMAN_OVERLAY_CALL_LOG=/tmp/foreman-overlay-calls.log
```

Add a test-only launch argument such as `--show-overlay-on-launch` so XCUITest
can bypass the global hotkey and directly validate UI behavior. Keep one manual
or local-only smoke for the actual global hotkey because CI often cannot grant
Accessibility/global-shortcut behavior reliably.

### Proposed repo tasks

```bash
mise run swift-check          # format/lint/build/test Swift package/app
mise run swift-test           # unit + contract tests
mise run swift-ui-test        # XCUITest with fake foreman binary
mise run verify-macos-overlay # fake + real tmux smoke on macOS only
```

CI should skip or mark macOS-only overlay tests clearly on non-macOS runners.
Real tmux + global hotkey tests should be opt-in/local until CI has a dedicated
macOS runner and permission model.

## Skill/Guide Work Before Implementation

Before coding the overlay, create lightweight repo-local guidance similar to the
Ratatui/TUI/UX skills Foreman already benefits from:

1. **Swift macOS overlay architecture guide**
   - AppKit windowing boundary, SwiftUI content boundary, observable store,
     dependency injection, process runner isolation.
2. **Swift overlay UX checklist**
   - Keyboard-first rules, states, accessibility, focus behavior, panel
     positioning, reduced motion, truncation, empty/error states.
3. **Swift overlay validation guide**
   - Fixture JSON, fake `foreman`, XCUITest launch args/env, snapshot policy,
     manual global-hotkey checklist, real tmux smoke.
4. **Foreman control API contract guide**
   - JSON schema compatibility, no stdout logs, stderr diagnostics,
     structured errors, versioning, fixture update process.

These can start as docs under `docs/` or a repo-local skill directory if the
implementation will be agent-heavy.

## Open Questions

- Should the Swift overlay live in this repo as a workspace member/`apps/macos`,
  or in a separate `foreman-macos` repo consuming a released `foreman` binary?
- Should the MVP use `MenuBarExtra`, a regular app with menu bar accessory, or
  both?
- Which minimum macOS version is acceptable? This affects `MenuBarExtra`,
  `@Observable`, and testing APIs.
- Which terminal app should the first activation helper target on this machine?
- Should `foreman agents --json` include preview text by default, or require an
  explicit flag for privacy/performance?
- Should Swift call `foreman` one-shot each refresh, or should Foreman later grow
  a local daemon/socket for low-latency updates?
