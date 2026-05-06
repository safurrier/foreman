---
id: macos-overlay-prototype-spec
title: Swift macOS Overlay Prototype Specification
description: Scope and acceptance criteria for the initial Foreman Swift overlay prototype plus validation/UX hardening.
---

# Specification — macos-overlay-prototype

## Problem

Foreman now has a promising native macOS overlay prototype, but its validation is
not yet comparable to Foreman's Rust/TUI/tmux validation ladder. We need to turn
this from a manually launched experiment into a repeatable UX iteration loop:
vendored macOS design guidance, Foreman-specific overlay skills, deterministic
fake data, and screenshot evidence.

## Requirements

### MUST

- Add or preserve a Foreman control API usable by external clients:
  - `foreman agents --json`
  - `foreman focus --pane <id> --json`
  - `foreman send --pane <id> --stdin --json`
- Keep control API output bounded so GUI clients do not hang on giant preview
  payloads.
- Provide a Swift overlay prototype that builds with SwiftPM.
- Vendor the two selected macOS design skills into repo-local `.agent/skills/`.
- Add Foreman-specific UX and validation skill wrappers for the Swift overlay.
- Add deterministic fake data and a fake-Foreman screenshot capture script.
- Store screenshot evidence under `.ai/validation/macos-overlay/`.

### SHOULD

- Add docs under `docs/macos-overlay/` for architecture, UX checklist, and
  validation ladder.
- Keep fake visual capture independent of live tmux state.
- Make future validation tasks easy to promote into `mise` tasks.

### NOT YET

- Full Swift XCTest/XCUITest suite.
- Configurable hotkey.
- Window-scoped screenshot capture.
- Packaged `.app` distribution.

## Acceptance Criteria

- `cargo test control_api --lib` passes.
- `swift build --package-path apps/macos-overlay` passes.
- `./scripts/capture-macos-overlay.sh` writes `.ai/validation/macos-overlay/attention.png`.
- `.agent/skills/macos-design-guidelines/SYNC.md` and `.agent/skills/macos-app-design/SYNC.md` record upstream source/sha.
- `docs/macos-overlay/validation.md` describes how Swift overlay validation maps to Foreman's existing validation ladder.
