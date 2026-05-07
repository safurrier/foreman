---
id: macos-swift-overlay-skill-blueprints
title: Proposed Skill / Guide Blueprints
description: >
  Candidate repo-local skills or docs to create before implementing the Swift
  macOS overlay.
---

# Skill / Guide Blueprints — macos-swift-overlay-speclab

These are not implemented skills yet. They are the recommended guidance units to
create before agents start writing Swift overlay code.

## 1. foreman-swift-overlay-architecture

**Trigger:** Working in `apps/macos-overlay/`, SwiftUI/AppKit overlay code,
`NSPanel`, hotkeys, or Foreman macOS companion app.

**Purpose:** Keep architecture boundaries intact.

**Core rules:**

- Use AppKit for overlay windowing: `NSPanel`, focus, positioning, Spaces/full
  screen behavior.
- Use SwiftUI for content: list/detail/compose/settings.
- Keep one observable store as source of truth.
- Views never construct `Process`, invoke `foreman`, run AppleScript, or know
  tmux syntax.
- `ForemanClient` talks only to the Foreman control API.
- `TerminalActivator` is optional and separate from `ForemanClient`.

**Must-read docs:**

- `docs/macos-overlay/architecture.md`
- `docs/control-api.md`
- `docs/architecture.md`

## 2. foreman-swift-overlay-ux

**Trigger:** Changing SwiftUI views, keyboard navigation, panel layout, search,
empty/loading/error states, compose flow.

**Purpose:** Preserve a Raycast/Spotlight-like control-plane feel without copying
Raycast’s product model.

**Core rules:**

- Global hotkey opens directly into Foreman, not into a generic launcher.
- Search is focused immediately.
- Keyboard-only happy path must work: open → filter/select → focus/send → close.
- Rows must show status, harness, workspace/session context, and attention state
  without visual clutter.
- Detail panel must distinguish status metadata from terminal preview.
- Loading, empty, broken foreman path, invalid JSON, and tmux unavailable states
  need explicit recovery copy.
- Avoid layout jumps when results update.
- Accessibility labels/focus order are part of done.

**Must-read docs:**

- `docs/macos-overlay/ux-checklist.md`
- `SPEC.md` status model and popup requirements

## 3. foreman-swift-overlay-validation

**Trigger:** Adding Swift code, changing app behavior, touching fake Foreman
harness, changing overlay validation tasks.

**Purpose:** Give Swift work the same validation discipline as Foreman’s Rust tmux
and native harness work.

**Core rules:**

- Most tests use fake `ForemanClient` or fake `ProcessRunner`.
- XCUITest uses a fake `foreman` executable and launch env/args.
- One focused real-tmux smoke is enough for the external seam.
- Do not require global hotkey permissions in CI.
- Store `.xcresult` bundles or summarized evidence under
  `.ai/validation/macos-overlay/`.
- Test launch mode must include `--show-overlay-on-launch` so UI tests can bypass
  global hotkeys.

**Must-read docs:**

- `docs/macos-overlay/validation.md`
- `docs/workflows.md`

## 4. foreman-control-api-contract

**Trigger:** Changing `foreman agents --json`, `foreman focus`, `foreman send`, or
Swift decoding models.

**Purpose:** Keep Rust and Swift in schema lockstep.

**Core rules:**

- JSON stdout is a public API. Logs and human diagnostics go to stderr.
- Include `schemaVersion`.
- Add/keep canonical fixtures for empty, working, attention, error, and mixed
  harness states.
- Swift decodes the same fixtures Rust tests produce or validate.
- Breaking schema changes require an explicit version bump and migration note.
- Pane ids are opaque strings; do not parse `%42` semantics in Swift.

**Must-read docs:**

- `docs/control-api.md`
- `.ai/plans/2026-05-03-140650-macos-popup-raycast-spike/SPEC.md`
