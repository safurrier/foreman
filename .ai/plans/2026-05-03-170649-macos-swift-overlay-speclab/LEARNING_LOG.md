---
id: macos-swift-overlay-speclab-learning-log
title: Learning Log
description: >
  Dev diary for the Swift macOS overlay speclab/research pass.
---

# Learning Log

## 2026-05-03

User clarified that Raycast may be too much of a launcher wrapper and asked for a
speclab-style external review of a simple native Swift macOS overlay before any
implementation.

Key findings:

- The product target is a Foreman-owned control-plane HUD, not just “Foreman as a
  Raycast command.” The UX should be one global hotkey directly into a native
  panel with search, rows, detail/preview, focus, and compose.
- Similar apps point toward a small command-palette anatomy: focused search,
  grouped/list rows, keyboard-first selection, right/secondary detail, explicit
  loading/error/empty states, and stable panel positioning.
- SwiftUI alone is not the right windowing layer. Use AppKit `NSPanel` for the
  overlay shell and host SwiftUI inside it.
- Use `KeyboardShortcuts` rather than a hardcoded hotkey library if this is meant
  to become daily-driver UX. It gives a recorder UI, persistence, conflict
  warnings, and sandbox compatibility.
- Validation should mirror Foreman’s Rust style: fake the external seam for most
  tests, then add one focused real seam smoke. For Swift, the seam is the
  `foreman` binary, so create fake process runners and fake-foreman scripts.
- Before implementation, create repo-local docs/skills for Swift overlay
  architecture, UX, validation, and control API contracts so coding agents have
  the same kind of rails Ratatui/TUI work has.
