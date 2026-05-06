# Code Context

## Files Retrieved
1. `apps/macos-overlay/Package.swift` (lines 1-25) - SwiftPM module/target boundaries: `ForemanOverlayCore`, `ForemanOverlayUI`, executable app, executable snapshot, tests.
2. `apps/macos-overlay/Sources/ForemanOverlay/main.swift` (lines 1-193) - panel, AppKit key equivalents, panel sizing/positioning, live panel snapshot/test hooks.
3. `apps/macos-overlay/Sources/ForemanOverlay/main.swift` (lines 195-376) - local key monitor, Carbon hotkey parsing/registration.
4. `apps/macos-overlay/Sources/ForemanOverlay/main.swift` (lines 378-497) - terminal activation adapter/factory.
5. `apps/macos-overlay/Sources/ForemanOverlay/main.swift` (lines 503-695) - settings SwiftUI view and settings panel controller.
6. `apps/macos-overlay/Sources/ForemanOverlay/main.swift` (lines 697-920) - `AppDelegate`, lifecycle, menu/status item, env config, scripted gauntlet.
7. `apps/macos-overlay/Sources/ForemanOverlayCore/Core.swift` (lines 1-225) - control API models, process seam, `ForemanClient` commands.
8. `apps/macos-overlay/Sources/ForemanOverlayCore/Core.swift` (lines 300-556) - filters, row presentation, preferences, keyboard command/effect interfaces.
9. `apps/macos-overlay/Sources/ForemanOverlayCore/Core.swift` (lines 558-940) - `OverlayStore` state, filtering/sorting, reducer, focus/send/reload actions.
10. `apps/macos-overlay/Sources/ForemanOverlayUI/OverlayView.swift` (lines 1-167) - overlay view root, search/content/list behavior.
11. `apps/macos-overlay/Sources/ForemanOverlayUI/OverlayView.swift` (lines 169-348) - detail/PR/footer/compose/preview behavior.
12. `apps/macos-overlay/Sources/ForemanOverlayUI/OverlayView.swift` (lines 350-520) - themes, row/status chips, help overlay.
13. `apps/macos-overlay/Sources/ForemanOverlaySnapshot/main.swift` (lines 1-220) - headless snapshot CLI, duplicated settings snapshot, fixture state registry.
14. `apps/macos-overlay/Tests/ForemanOverlayCoreTests/ForemanOverlayCoreTests.swift` (lines 1-260) - pure-ish store/client/process tests.
15. `apps/macos-overlay/Tests/ForemanOverlayUITests/ForemanOverlayUITests.swift` (lines 1-35) - shell-backed UI gauntlet XCTest wrapper.
16. `scripts/macos-overlay-gauntlet.sh` (lines 1-132) - fake Foreman executable, env-driven app launch, scripted call assertions.
17. `scripts/render-macos-overlay-snapshots.sh` (lines 1-29) - snapshot render wrapper and hardcoded state assertions.
18. `scripts/verify-macos-overlay.sh` (lines 1-45) - full overlay validation ladder and another hardcoded state list.
19. `docs/macos-overlay/architecture.md` (lines 13-30, 64-68) - intended boundaries and explicit note to extract settings/lifecycle.
20. `docs/macos-overlay/validation.md` (lines 13-25, 94-110, 150-180) - intended validation ladder, hotkey/terminal notes, gauntlet targets.

## Key Code

### Current target/module shape is broad but shallow
```swift
// apps/macos-overlay/Package.swift, lines 9-24
.library(name: "ForemanOverlayCore", targets: ["ForemanOverlayCore"]),
.library(name: "ForemanOverlayUI", targets: ["ForemanOverlayUI"]),
.executable(name: "foreman-overlay", targets: ["ForemanOverlay"]),
.executable(name: "foreman-overlay-snapshot", targets: ["ForemanOverlaySnapshot"]),
```
`Core.swift`, `OverlayView.swift`, `main.swift`, and snapshot `main.swift` are each large single-file modules/targets. The nominal modules exist, but most interfaces are file-private concrete implementations rather than narrow importable seams.

### Good seam: process runner / control API client
```swift
// apps/macos-overlay/Sources/ForemanOverlayCore/Core.swift, lines 74-75
public protocol ProcessRunning: Sendable {
    func run(_ executable: String, _ arguments: [String], stdin: String?) async throws -> ProcessResult
}
```
```swift
// apps/macos-overlay/Sources/ForemanOverlayCore/Core.swift, lines 200-220
public func agents() async throws -> AgentsResponse { ... ["agents", "--json", "--pull-requests"] ... }
public func focus(paneId: String) async throws { ... ["focus", "--pane", paneId, "--json"] ... }
public func send(paneId: String, text: String) async throws { ... ["send", "--pane", paneId, "--stdin", "--json"] ... }
```
This is the deepest/highest-leverage interface in the overlay: typed, fakeable, and tested with `FakeRunner` in `ForemanOverlayCoreTests.swift` lines 4-17 and client tests around lines 257+.

### Main executable is a locality hot spot
`apps/macos-overlay/Sources/ForemanOverlay/main.swift` contains:
- panel and panel snapshot/test hooks (lines 16-193)
- local keyboard monitor (lines 195-274)
- separate Carbon hotkey implementation (lines 276-376)
- terminal activation adapter/factory (lines 378-497)
- full settings UI/panel (lines 503-695)
- `AppDelegate`, lifecycle, menus, status item, env config, scripted gauntlet (lines 697-920)

This makes `main.swift` the highest-friction place for deepening: many responsibilities are adjacent but not separated by importable interfaces.

### Keyboard seam is partially deep, partially leaky
```swift
// apps/macos-overlay/Sources/ForemanOverlayCore/Core.swift, lines 532-556
public enum OverlayKeyboardCommand { ... }
public enum OverlayKeyboardEffect { ... }
```
```swift
// apps/macos-overlay/Sources/ForemanOverlay/main.swift, lines 217-249
switch Int(event.keyCode) { ... command = .enter(command: hasCommand) ... }
...
switch store.handleKeyboardCommand(command) { ... }
```
The reducer is testable in Core (`ForemanOverlayCoreTests.swift` lines 173-239), but key mapping is split between `ForemanOverlayPanel.performKeyEquivalent` (`main.swift` lines 20-42) and `OverlayKeyboardController.handle` (`main.swift` lines 209-272). Example: Cmd+T cycles theme only in `performKeyEquivalent`; `OverlayKeyboardCommand.cycleTheme` exists in Core but no key monitor branch emits it. Cmd+, is panel-level for command-comma, while Core also treats typed comma as `.openSettings` (`Core.swift` lines 860-863), which is not the same interface.

### Store is both model and action dispatcher
```swift
// apps/macos-overlay/Sources/ForemanOverlayCore/Core.swift, lines 560-589
@Published public var query = ""
@Published public var response: AgentsResponse?
...
public var onFocusSucceeded: (() -> Void)?
public var onOpenSettings: (() -> Void)?
public var onOpenURL: ((String) -> Void)?
public let preferences: OverlayPreferences
private var client: ForemanClient
```
`OverlayStore` has useful depth for query/filter/sort/keyboard state, but it also owns async `reload`, `focusSelected`, `sendToSelected`, PR URL opening callbacks, and settings callbacks. The interface exposes app concerns back upward through closures, so Core has indirect knowledge of settings/browser/panel lifecycle.

### UI imports AppKit and can hide windows
```swift
// apps/macos-overlay/Sources/ForemanOverlayUI/OverlayView.swift, lines 293-295
.onKeyPress(.escape) {
    NSApp.keyWindow?.orderOut(nil)
    return .handled
}
```
This violates the documented boundary that SwiftUI views render state and send intents (`docs/macos-overlay/architecture.md` lines 20-21). It is a direct AppKit side effect from UI, bypassing `OverlayKeyboardEffect.hideOverlay` and `OverlayPanelController.hide()`.

### Snapshot target duplicates settings implementation
```swift
// apps/macos-overlay/Sources/ForemanOverlaySnapshot/main.swift, lines 105-168
struct SettingsSnapshotView: View { ... Text("Ctrl+F") ... }
```
The real settings view lives in executable `main.swift` (`main.swift` lines 511-655), not in an importable module. Snapshot cannot reuse it, so settings coverage is shallow: it validates a hand-made facsimile, not the actual settings panel/hotkey UI.

## Architecture

Intended flow from docs is:

```text
SwiftUI views → OverlayStore → ForemanClient → ProcessRunner → foreman binary → tmux
             ↘ OverlayPanelController / HotkeyController at app boundary only
```
See `docs/macos-overlay/architecture.md` lines 13-30.

Actual flow mostly follows that for data loading/actions, but several seams leak:

1. **Core data/control path** is the strongest part. `ForemanClient` is typed and depends on the `ProcessRunning` protocol. Unit tests fake the process layer and fixture-decode the Rust JSON contract.
2. **Store/UI path** is less local. `OverlayView` calls store methods directly for focus/reload/send/settings and also performs an AppKit hide. It is not a pure intent-emitting view layer.
3. **AppKit/app lifecycle path** is concentrated in `main.swift`. Panel, hotkey, terminal activation, settings, menus, app activation/reopen, env flags, fake-gauntlet hooks, and status item behavior are colocated in one executable-only file. This keeps local editing easy for prototype work but gives low module depth: callers cannot depend on small interfaces for settings, hotkeys, terminal activation, or lifecycle.
4. **Snapshot/test path** uses the importable Core/UI modules, but cannot import the real app/settings/hotkey code. It therefore duplicates settings UI and drives app behavior with env variables, sleeps, hardcoded mouse coordinates, and shell fake Foreman scripts.

## Friction Points / Deepening Opportunities

### 1. `main.swift` is a shallow mega-module with many unrelated implementations
- Files: `apps/macos-overlay/Sources/ForemanOverlay/main.swift` lines 16-920; `docs/macos-overlay/architecture.md` lines 64-68.
- Why shallow/leaky: the executable target is acting as module, interface, and implementation for panel, settings, hotkey, terminal activation, lifecycle, status menus, and testing hooks. Most types are not public/importable, so tests and snapshot cannot exercise them directly.
- Terms: **Module** boundaries exist at SwiftPM target level, but `ForemanOverlay` is a single implementation blob. **Locality** is poor when changing settings/hotkey/lifecycle because unrelated code is in one file.

### 2. Settings is not an importable module; snapshot uses a separate implementation
- Files: real settings in `apps/macos-overlay/Sources/ForemanOverlay/main.swift` lines 503-695; fake settings snapshot in `apps/macos-overlay/Sources/ForemanOverlaySnapshot/main.swift` lines 105-168.
- Why shallow/leaky: `settings-general` screenshot validates `SettingsSnapshotView`, not `SettingsView`. Hotkey recorder (`KeyboardShortcuts.Recorder`) appears only in real settings (`main.swift` lines 560-564), while snapshot hardcodes `Text("Ctrl+F")` (`Snapshot/main.swift` lines 130-140).
- Risk: settings regressions can pass snapshots because the snapshot target covers a parallel UI.

### 3. Hotkey behavior has multiple adapters and defaults
- Files: `main.swift` lines 9-13, 276-376, 739-752; `docs/macos-overlay/validation.md` lines 99-105.
- Why shallow/leaky: there are two hotkey systems: `KeyboardShortcuts` for normal persisted hotkey and a Carbon `HotkeyController` for `FOREMAN_OVERLAY_HOTKEY`. `HotkeySpec.userDefaultsKey = "ForemanOverlayHotkey"` is not obviously the same storage as `KeyboardShortcuts.Recorder`. Docs say default persisted hotkey is `Ctrl+Option+Shift+A` (`validation.md` lines 101-105), while code and architecture docs say Ctrl+F (`main.swift` lines 9-13, 281; `architecture.md` line 54).
- Risk: env/testing path and user settings path diverge; status item label update is handled explicitly on reset (`main.swift` lines 945-ish in read context) but not obviously as a general recorder-change observer.

### 4. Keyboard interface is duplicated across panel and local monitor
- Files: `main.swift` lines 20-42 and 209-272; `Core.swift` lines 532-556 and 800-923.
- Why shallow/leaky: Core defines a reducer interface, but not a single key-event adapter. Command shortcuts are handled in both AppKit panel key equivalents and local key monitor. Some commands exist in Core but are emitted only from panel-level code (`cycleTheme`), and some app actions are encoded as typed characters (`typed(",")`) rather than command-key semantics.
- Risk: adding/changing shortcuts requires checking three places: panel, monitor, store reducer/help text.

### 5. Terminal activation has a protocol but not a testable/importable adapter module
- Files: `main.swift` lines 378-497 and 725-731; `Core.swift` lines 489-504; `docs/macos-overlay/validation.md` lines 106-148.
- Why shallow/leaky: `TerminalActivating` is a good seam name, but it lives in executable `main.swift` and depends directly on `NSWorkspace`. Preferences store terminal activation as raw strings in Core (`Core.swift` lines 489-504), and `AppDelegate` recreates a factory from preferences on focus (`main.swift` lines 725-731). The `terminalActivator` property initialized at launch is mostly bypassed by this recomputation.
- Risk: behavior depends on installed/running apps and is only shell/manual covered; there is little leverage for unit-level validation of parsing/factory behavior.

### 6. `OverlayStore` is deep in state handling but too broad as an interface
- Files: `Core.swift` lines 558-940.
- Why shallow/leaky: the store has useful depth for selection, filters, flash, help, and keyboard reducer; however it also owns async process-backed actions (`reload`, `focusSelected`, `sendToSelected`) and app callbacks (`onOpenSettings`, `onOpenURL`, `onFocusSucceeded`). This makes it an implementation hub rather than a crisp interface between UI state and app commands.
- Risk: UI, app shell, and process client all depend on the same mutable object. Adding new capabilities tends to widen `OverlayStore` rather than create local feature seams.

### 7. `OverlayView.swift` is a single large view module with direct store/app actions
- Files: `OverlayView.swift` lines 1-520.
- Why shallow/leaky: the file contains root view, list, details, PR panel, footer, compose input, preview scroller, row, chips, help, theme mapping, and color helpers. It calls `store.focusSelected`, `store.reload`, `store.sendToSelected`, and `store.onOpenSettings` directly (`OverlayView.swift` lines 258-275, 299-310), and calls `NSApp.keyWindow?.orderOut` directly (`lines 293-295`).
- Risk: visual/layout changes can touch behavior, and behavior changes can require scanning a large SwiftUI file. The view is not just an implementation of a narrow presentation interface.

### 8. Snapshot state registry and fixture expectations are duplicated
- Files: `ForemanOverlaySnapshot/main.swift` lines 52-91; `scripts/render-macos-overlay-snapshots.sh` lines 13-27; `scripts/verify-macos-overlay.sh` lines 23-28; `scripts/verify-macos-overlay-snapshot-text.swift` lines 8-25; `docs/macos-overlay/validation.md` lines 39-52.
- Why shallow/leaky: snapshot states are hardcoded in multiple locations. `render-macos-overlay-snapshots.sh` only manually checks a subset of states (omits `pr-active`, `duplicate-workspace`, `compose`, `flash`, `settings-general` in its shell loop), while `verify-macos-overlay.sh` expects all. Docs' expected artifact tree is also stale and omits newer states.
- Risk: adding/removing snapshot states requires multi-file edits; validation/doc drift is already visible.

### 9. Gauntlet/test fixtures cross seams through environment and sleeps
- Files: `main.swift` lines 122-148 and 872-895; `scripts/macos-overlay-gauntlet.sh` lines 23-115; `ForemanOverlayUITests.swift` lines 4-33.
- Why shallow/leaky: the test path posts hardcoded mouse coordinates (`main.swift` lines 122-148), waits fixed durations (`main.swift` lines 875-893), launches a fake executable through env vars (`gauntlet.sh` lines 67-74), and asserts log lines/call counts (`gauntlet.sh` lines 89-115). It proves end-to-end smoke, but not through stable app-level interfaces.
- Risk: fragile timings/coordinates; hard to validate app lifecycle/hotkey/settings behavior independently.

### 10. Environment configuration is scattered
- Files: panel env reads in `main.swift` lines 78-82, 94-105, 163-181; app/env reads in lines 713-766, 779-807; terminal env reads in lines 466-477; scripts `capture-macos-overlay.sh` and `macos-overlay-gauntlet.sh`.
- Why shallow/leaky: env vars are read directly at use sites rather than through one typed configuration adapter. That makes behavior discoverability low and complicates test setup.
- Risk: scripts and app can drift; tests may unintentionally exercise different combinations of flags than real app launches.

### 11. Documentation reflects desired boundaries but also current drift
- Files: `docs/macos-overlay/architecture.md` lines 18-30, 64-68; `docs/macos-overlay/validation.md` lines 94-110 and 150-180.
- Why shallow/leaky: architecture docs already call out extracting settings/lifecycle later (`architecture.md` line 68). Validation docs say hotkey default `Ctrl+Option+Shift+A`, but code/docs architecture say `Ctrl+F`. Validation docs' current gaps section (later in file) also appears stale relative to implemented help/region/snapshot behavior.
- Risk: agents following docs may make changes against outdated constraints.

## Start Here

Start with `apps/macos-overlay/Sources/ForemanOverlay/main.swift`. It is where the shallowest modules and leakiest seams converge: settings, hotkey, terminal activation, panel lifecycle, app lifecycle, menus, env config, and scripted testing all live in one executable-only file. After that, open `apps/macos-overlay/Sources/ForemanOverlayCore/Core.swift` to see which behavior already has a deeper interface (`ProcessRunning`, `ForemanClient`, keyboard reducer) and which app concerns have leaked back into Core (`OverlayStore` callbacks/preferences).