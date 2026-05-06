---
id: macos-overlay-prototype-learning-log
title: Learning Log
description: Dev diary for Swift macOS overlay prototype and validation/UX hardening.
---

# Learning Log

## 2026-05-04

Built the first Swift overlay prototype and launched it against local Foreman.
The first manual run showed an endless spinner. Investigation found that
`foreman agents --json` was returning roughly 249 KB because pane previews were
unbounded. The Swift process wrapper waited for process termination before
reading stdout, so the child could block on a full pipe. Added Rust-side preview
truncation in the control API, which brought the payload down to roughly 21 KB
and made the overlay render. This is a product/API improvement but not a full
subprocess hardening fix; the Swift `ProcessRunner` still needs async stdout and
stderr draining.

User found two external macOS design skills. Vendored both into repo-local
`.agent/skills/` using the skill curator script:

- `macos-design-guidelines` from `ehmo/platform-design-skills/skills/macos`
- `macos-app-design` from `petekp/agent-skills/skills/macos-app-design`

Added Foreman-specific wrapper skills so future agents know when and how to use
the vendored guidance while working on this overlay:

- `foreman-swift-overlay-ux`
- `foreman-swift-overlay-validation`

Added the first deterministic visual capture lane. `scripts/capture-macos-overlay.sh`
creates a fake `foreman`, launches the overlay with a checked-in attention
fixture, and saves a screenshot under `.ai/validation/macos-overlay/attention.png`.
This gives us the start of a Foreman-style UX artifact loop for the Swift overlay.
