---
name: foreman-swift-overlay-validation
description: Use when adding tests, fixtures, screenshot capture, fake foreman harnesses, or validation tasks for Foreman's Swift macOS overlay.
---

# Foreman Swift Overlay Validation

Use this skill when changing the Swift overlay validation lane or before calling
macOS overlay work done.

## Load First

Read:

1. `docs/macos-overlay/validation.md`
2. `docs/macos-overlay/architecture.md`
3. `docs/workflows.md`

## Validation Philosophy

Mirror Foreman's Rust validation ladder:

- fast pure tests first
- fake external seams before real external seams
- deterministic visual artifacts for UX review
- one focused real tmux smoke for integration confidence
- manual-only checks for global hotkeys and macOS permissions

## Required Proof by Change Type

| Change | Minimum proof |
|---|---|
| Control API schema | Rust tests + Swift fixture decode once available |
| Swift model/filtering | Swift unit tests once test target exists |
| Process invocation | fake `ProcessRunner` or fake `foreman` call-log assertion |
| Visual/layout change | `./scripts/capture-macos-overlay.sh` and screenshot review |
| Focus/send behavior | fake call log + real tmux smoke if adapter semantics changed |
| Hotkey/window behavior | local manual checklist; don't require CI hotkey synthesis |

## Screenshot Capture

Use:

```bash
./scripts/capture-macos-overlay.sh
```

Artifacts land in:

```text
.ai/validation/macos-overlay/
```

Do not use live tmux state for visual review fixtures. Use checked-in fixture JSON
under `apps/macos-overlay/Fixtures/`.

## Current Known Gap

The prototype currently has build/manual/screenshot proof, but not Swift
XCTest/XCUITest proof. Do not claim Foreman-grade validation until those are
added.
