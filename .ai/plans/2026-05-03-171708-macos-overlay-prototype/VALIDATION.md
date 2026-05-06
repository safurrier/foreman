---
id: macos-overlay-prototype-validation
title: Validation Log
description: Evidence gathered for Swift macOS overlay prototype and validation/UX hardening.
---

# Validation — macos-overlay-prototype

## 2026-05-04 — Current proof

Build/unit commands run:

```bash
cargo test control_api --lib
swift build --package-path apps/macos-overlay
```

Manual/control API smoke run earlier:

- `foreman agents --json` against a temp tmux socket returned schema v1 agent data.
- `foreman focus --pane %0 --json` returned success JSON.
- `foreman send --pane %0 --stdin --json` returned success JSON and pasted into the tmux pane.

Prototype UX smoke:

- Launched `apps/macos-overlay/.build/debug/foreman-overlay` with:

```bash
FOREMAN_OVERLAY_FOREMAN_PATH=$PWD/target/debug/foreman \
FOREMAN_OVERLAY_SHOW_ON_LAUNCH=1 \
apps/macos-overlay/.build/debug/foreman-overlay
```

- User confirmed the overlay rendered successfully after preview output was bounded.
- Initial failure mode: overlay spinner appeared endless because the Swift process wrapper waited for child termination before draining a large JSON stdout pipe. Rust-side preview truncation reduced payload size and made the prototype usable; Swift-side process draining still needs hardening.

## 2026-05-04 — Design skill vendoring

Vendored skills:

```bash
python3 ~/.pi/agent/skills/alex-ai-skill-curator/resources/scripts/vendor_skill.py \
  --dest .agent/skills vendor \
  --source https://github.com/ehmo/platform-design-skills \
  --path skills/macos \
  --name macos-design-guidelines

python3 ~/.pi/agent/skills/alex-ai-skill-curator/resources/scripts/vendor_skill.py \
  --dest .agent/skills vendor \
  --source https://github.com/petekp/agent-skills \
  --path skills/macos-app-design \
  --name macos-app-design
```

Installed:

- `.agent/skills/macos-design-guidelines/`
- `.agent/skills/macos-app-design/`

Added Foreman-specific wrappers:

- `.agent/skills/foreman-swift-overlay-ux/SKILL.md`
- `.agent/skills/foreman-swift-overlay-validation/SKILL.md`

## 2026-05-04 — Deterministic visual capture

Command run:

```bash
./scripts/capture-macos-overlay.sh
mise run capture-macos-overlay
```

Artifacts:

- `.ai/validation/macos-overlay/attention.png`
- `.ai/validation/macos-overlay/fake-foreman-calls.log`
- `.ai/validation/macos-overlay/fake-foreman-stdin.log`
- `.ai/validation/macos-overlay/foreman-overlay.log`
- `.ai/validation/macos-overlay/summary.md`

The fake call log currently shows:

```text
agents --json
```

This proves the overlay can be launched in an isolated deterministic state and
captured for visual UX iteration without relying on live tmux inventory.

## 2026-05-04 — Hardening proof

Commands run:

```bash
swift test --package-path apps/macos-overlay
./scripts/smoke-control-api-tmux.sh
mise run verify-macos-overlay
mise run check
```

Results:

- Swift tests passed: fixture decoding, filtering, selection movement, fake Foreman client decode/failure behavior, and large stdout drain regression.
- Real tmux smoke passed: isolated tmux socket, `agents --json`, `focus --pane`, and `send --stdin` with pane capture assertion.
- Full macOS overlay verifier passed and wrote `.ai/validation/macos-overlay/verify-summary.md`.
- Repo fast gate `mise run check` passed after rerun with a longer timeout.
- Manual real overlay was relaunched against `target/debug/foreman` and user feedback drove double-click focus plus command-palette keyboard behavior.

## 2026-05-04 — Overlay gauntlet slice

Commands run:

```bash
swift build --package-path apps/macos-overlay
swift test --package-path apps/macos-overlay
mise run macos-overlay-gauntlet
mise run verify-macos-overlay
```

Results:

- Swift tests passed: 9 tests now cover fixture decode, filtering, selection,
  region cycling, help/compose cancel behavior, preview scroll clamping, client
  errors, and large stdout drain.
- Scripted fake-Foreman gauntlet passed and wrote
  `.ai/validation/macos-overlay/gauntlet/summary.md`.
- `verify-macos-overlay` now includes the fake-Foreman action gauntlet, real tmux
  smoke, Swift tests, Rust control API tests, and multi-state screenshot
  capture.
- Multi-state screenshots were generated under
  `.ai/validation/macos-overlay/screenshots/` for attention, empty, error,
  long-path, long-preview, many, and mixed-status.

## 2026-05-04 — Headless snapshot lane

Commands run:

```bash
swift build --package-path apps/macos-overlay
swift test --package-path apps/macos-overlay
mise run render-macos-overlay-snapshots
mise run verify-macos-overlay
```

Results:

- SwiftUI overlay views were split into `ForemanOverlayUI` so the real app and
  snapshot renderer share the same UI code.
- Added `foreman-overlay-snapshot`, an offscreen `NSHostingView` renderer that
  writes PNGs without showing the floating panel.
- Headless snapshots passed for attention, empty, error, long-path,
  long-preview, many, mixed-status, help, and help-scrolled states.
- `verify-macos-overlay` now uses headless snapshots instead of live panel
  screenshot capture, so routine verification does not interrupt the active
  desktop.

## 2026-05-04 — Reducer/config/activation slice

Commands run:

```bash
swift build --package-path apps/macos-overlay
swift test --package-path apps/macos-overlay
```

Results:

- Keyboard behavior now routes through a testable `OverlayKeyboardCommand` /
  `OverlayKeyboardEffect` reducer in `ForemanOverlayCore`.
- Swift tests now cover search typing, delete, Enter focus effect, help/detail
  scrolling commands, compose pass-through/send behavior, and Escape cancel.
- Added `FOREMAN_OVERLAY_HOTKEY` parsing for environment-configurable hotkeys;
  default remains `Ctrl+Option+Cmd+F`.
- Added `TerminalActivating` adapter boundary. Default is no terminal activation;
  optional values include `terminal`, `iterm`, `ghostty`, `wezterm`, or
  `bundle:<bundle-id>`.

## 2026-05-04 — Remaining-gap burn-down

Commands run:

```bash
swift build --package-path apps/macos-overlay
swift test --package-path apps/macos-overlay
mise run macos-overlay-gauntlet
mise run render-macos-overlay-snapshots
mise run verify-macos-overlay
```

Results:

- Live panel capture now asks the app to write a panel-content PNG directly via
  `FOREMAN_OVERLAY_PANEL_SNAPSHOT_PATH`, avoiding `screencapture -l` failures
  for routine live capture artifacts.
- Added menu-bar Settings surface for hotkey editing. The local parser supports
  modifiers plus letters, numbers, and `space`; settings persist to
  `UserDefaults` and re-register the hotkey live.
- Scripted gauntlet now posts app key events for Enter focus and Cmd+Enter send;
  it still seeds compose state internally because full compose focus event
  synthesis is not stable enough yet.
- Added OCR-based semantic snapshot assertions using Vision. Headless snapshot
  verification now fails if expected text is not recognized in the rendered PNGs.

## 2026-05-04 — Final validation polish burn-down

Commands run:

```bash
swift build --package-path apps/macos-overlay
swift test --package-path apps/macos-overlay
mise run macos-overlay-gauntlet
mise run verify-macos-overlay
```

Results:

- Added `KeyboardShortcuts` package and replaced the normal persisted hotkey UI
  with `KeyboardShortcuts.Recorder`. `FOREMAN_OVERLAY_HOTKEY` remains as a
  development override path.
- Fake-Foreman gauntlet now includes synthesized mouse/key-event focus coverage
  plus Cmd+Enter send coverage and terminal-activation missing-app non-fatal
  logging.
- Terminal activation missing-app behavior is now covered automatically; real
  installed terminal foregrounding remains a manual OS smoke because it depends
  on local apps and macOS focus policy.

## 2026-05-04 — UI XCTest + compose typing closeout

Commands run:

```bash
mise run macos-overlay-ui-tests
mise run verify-macos-overlay
```

Results:

- Added `ForemanOverlayUITests` SwiftPM XCTest target. It is skipped during plain
  `swift test` unless `FOREMAN_OVERLAY_RUN_UI_TESTS=1` is set.
- Added `mise run macos-overlay-ui-tests` and folded it into
  `verify-macos-overlay`.
- UI event gauntlet now types compose text through posted key events before
  posting Cmd+Enter, rather than seeding compose text directly.
- `verify-macos-overlay` passed with the UI event XCTest target enabled.

## 2026-05-04 — PR link, themes, and default hotkey

Commands run:

```bash
cargo fmt
swift build --package-path apps/macos-overlay
swift test --package-path apps/macos-overlay
cargo test control_api --lib
mise run verify-macos-overlay
```

Results:

- Added optional pull-request metadata to the control API with
  `foreman agents --json --pull-requests`.
- Swift overlay now requests pull-request metadata and renders a PR detail panel
  with `Open PR` link when a workspace has an associated PR.
- Fixture snapshots include PR metadata; OCR asserts `Pull Request` and `Open PR`
  are visible.
- Default global hotkey changed to `Ctrl+Option+Shift+A`; `KeyboardShortcuts`
  settings and `FOREMAN_OVERLAY_HOTKEY` override remain available.
- Added lightweight themes (`System`, `Graphite`, `Indigo`, `Terminal`) and
  `Cmd+T` theme cycling. Headless snapshots now include `theme-indigo` and
  `theme-terminal`.
- Full overlay verifier passed with 11 OCR-checked snapshot states.
- Design/UX review recorded at `.ai/validation/macos-overlay/design-ux-review.md`, including PR card, theme, footer density, and terminal activation follow-ups.

## Remaining validation gaps

- This is a SwiftPM XCTest UI-event gauntlet rather than an Xcode-managed
  `XCUIApplication` UI test bundle. It still validates the running app via
  event synthesis and fake-Foreman logs.
- Real terminal foregrounding still needs manual confirmation against the user's
  installed terminal apps.
- Pull-request lookup is best-effort and synchronous in the control command;
  if `gh pr view` is slow or unauthenticated, the overlay simply omits the PR
  card for that workspace.
