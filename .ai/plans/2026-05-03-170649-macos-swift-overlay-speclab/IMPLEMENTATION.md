---
id: macos-swift-overlay-speclab-implementation
title: Swift macOS Overlay Research Implementation Plan
description: >
  Sequenced pre-implementation plan for building guidance, validation harnesses,
  and then a minimal native Swift Foreman overlay.
---

# Implementation — macos-swift-overlay-speclab

## Principle

Do not start with the full app. Start by creating the contracts and agent-safe
rails that make the app easy to implement without architectural drift.

Recommended sequence:

1. Foreman JSON/control API contract.
2. Swift overlay docs/skills/validation harness.
3. Tiny Swift spike behind fake Foreman CLI.
4. Real tmux smoke.
5. Only then polish/global-hotkey/terminal activation behavior.

## Phase 0 — Decide repo shape

### Option A: Same repo, `apps/macos-overlay/`

Pros:

- Keeps control API and native client co-evolving.
- Easier to share fixtures and validation tasks.
- Good while the product is experimental.

Cons:

- Adds Swift/Xcode project surface to a Rust repo.
- CI/mise setup becomes more platform-specific.

### Option B: Separate `foreman-macos` repo

Pros:

- Cleaner release/app packaging boundary.
- Avoids Rust repo dependency/toolchain bloat.

Cons:

- Harder to keep JSON fixtures/contracts synchronized early.
- More overhead before we know the overlay is useful.

### Recommendation

Start in-repo under `apps/macos-overlay/` while experimental. Revisit extraction
once the overlay has a release story or public audience.

## Phase 1 — Build pre-implementation guides/skills

Create durable docs before implementation:

```text
docs/macos-overlay/architecture.md
docs/macos-overlay/ux-checklist.md
docs/macos-overlay/validation.md
docs/control-api.md
```

Or, if you prefer executable agent guidance, create repo-local skills mirroring
those docs:

```text
.agent/skills/foreman-swift-overlay-architecture/SKILL.md
.agent/skills/foreman-swift-overlay-ux/SKILL.md
.agent/skills/foreman-swift-overlay-validation/SKILL.md
.agent/skills/foreman-control-api-contract/SKILL.md
```

These should be short, concrete, and file/path-oriented. They should tell agents
what not to do: no `Process` in views, no AppKit windowing in SwiftUI rows, no
terminal activation in `ForemanClient`, no global hotkey reliance in CI.

## Phase 2 — Foreman control API prerequisite

The Swift overlay should not scrape TUI output. It needs:

```bash
foreman agents --json
foreman focus --pane %42 --json
foreman send --pane %42 --stdin --json
```

The JSON schema should be versioned and fixture-backed. Keep a canonical fixture
set under something like:

```text
tests/fixtures/control-api/agents-empty.json
tests/fixtures/control-api/agents-attention.json
tests/fixtures/control-api/agents-working.json
```

Swift tests should decode those same fixtures so Rust and Swift stay aligned.

## Phase 3 — Swift package/app skeleton

Suggested structure:

```text
apps/macos-overlay/
  Package.swift or ForemanOverlay.xcodeproj
  Sources/ForemanOverlayApp/
    ForemanOverlayApp.swift
    AppDependencies.swift
    OverlayPanelController.swift
    HotkeyController.swift
    TerminalActivator.swift
  Sources/ForemanOverlayCore/
    OverlayStore.swift
    ForemanClient.swift
    ProcessRunner.swift
    AgentFilter.swift
    Models.swift
  Sources/ForemanOverlayViews/
    OverlayView.swift
    AgentRowView.swift
    AgentDetailView.swift
    ComposeView.swift
    SettingsView.swift
  Tests/ForemanOverlayCoreTests/
  Tests/ForemanOverlayViewTests/
  Tests/ForemanOverlayUITests/
  Tests/Fixtures/
```

Keep `ForemanOverlayCore` free of AppKit when possible. AppKit/SwiftUI code can
then be thinner and less risky.

## Phase 4 — Fake Foreman harness

Implement test utilities before real UI polish:

- `FakeProcessRunner`: unit-level fake returning stdout/stderr/status.
- `fake-foreman` script: XCUITest-level fake executable.
- `FixtureForemanClient`: preview/dev mode client using bundled JSON.
- Call-log assertions: verify `focus --pane %N` and `send --pane %N --stdin`.

This gives the Swift side the same confidence pattern Foreman has for tmux
adapters: most tests use fakes, one focused smoke crosses the real seam.

## Phase 5 — Minimal overlay UX

Implement the smallest complete loop:

1. App starts as menu bar utility.
2. Test launch arg `--show-overlay-on-launch` opens overlay immediately.
3. Hotkey toggles overlay in manual/local use.
4. Overlay loads agents through `ForemanClient.agents()`.
5. Search filters rows.
6. Selection detail shows preview.
7. Enter calls `focus`.
8. Compose calls `send`.
9. Errors show inline + menu action to run doctor/copy command.

## Phase 6 — Validation tasks

Add mise tasks only after the app skeleton exists:

```toml
[tasks.swift-test]
run = "xcodebuild test -scheme ForemanOverlay -destination 'platform=macOS' -resultBundlePath .ai/validation/macos-overlay/unit.xcresult"

[tasks.swift-ui-test]
run = "xcodebuild test -scheme ForemanOverlayUITests -destination 'platform=macOS' -resultBundlePath .ai/validation/macos-overlay/ui.xcresult"

[tasks.verify-macos-overlay]
run = "scripts/verify-macos-overlay.sh"
```

The verification script should:

- Build the Rust `foreman` binary.
- Build/test Swift unit tests.
- Run XCUITest against fake `foreman`.
- Optionally start a tmux fixture and run a local real smoke on macOS.
- Write a summary under `.ai/validation/macos-overlay/`.

## Phase 7 — Manual UX checklist

Before calling the overlay usable, manually verify:

- Hotkey opens from Finder, browser, editor, and terminal.
- Overlay opens on focused display.
- Search field is focused immediately.
- Escape dismisses reliably.
- Enter focuses tmux pane.
- Terminal app foregrounds, or product explicitly documents that it does not.
- Full-screen app / Spaces behavior is acceptable.
- Long output, long paths, duplicate names, empty tmux, and broken foreman path are
  readable and recoverable.
- VoiceOver/focus labels are minimally sane.

## Decision Points

- **KeyboardShortcuts vs HotKey**: choose KeyboardShortcuts for user-configurable
  shortcuts unless the first spike is intentionally hardcoded and personal-only.
- **MenuBarExtra minimum OS**: if macOS 13+ is acceptable, use it. Otherwise use
  a classic `NSStatusItem`.
- **Observation model**: if macOS 14+ is acceptable, `@Observable` is clean. If
  not, use `ObservableObject`/`@Published`.
- **Terminal activation**: keep optional and isolated behind `TerminalActivator`.
  Do not mix it into `ForemanClient`.
