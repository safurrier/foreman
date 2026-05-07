---
id: macos-swift-overlay-speclab-todo
title: Task List
description: >
  Checklist for the Swift macOS overlay speclab/research pass and future work.
---

# TODO — macos-swift-overlay-speclab

## Speclab Pass

- [x] Create plan directory.
- [x] Research similar macOS launcher/command-palette UX patterns.
- [x] Research Swift global hotkey and floating panel implementation options.
- [x] Research Swift architecture/testing recommendations.
- [x] Review Foreman's existing validation ladder for analogues.
- [x] Draft Swift overlay product spec.
- [x] Draft Swift architecture and component boundaries.
- [x] Draft Swift/agent-harness validation strategy.
- [x] Identify repo-local skill/guide work to create before implementation.
- [x] Draft skill/guide blueprints for architecture, UX, validation, and control API contracts.

## Pre-Implementation Docs / Skills

- [ ] Write `docs/macos-overlay/architecture.md`.
- [ ] Write `docs/macos-overlay/ux-checklist.md`.
- [ ] Write `docs/macos-overlay/validation.md`.
- [ ] Write `docs/control-api.md` for Foreman JSON/control API contracts.
- [ ] Decide whether to also encode those as repo-local skills under `.agent/skills/`.

## Control API Prerequisite

- [ ] Define versioned `foreman agents --json` schema.
- [ ] Add canonical JSON fixtures for empty/attention/working states.
- [ ] Implement `foreman focus --pane ... --json`.
- [ ] Implement `foreman send --pane ... --stdin --json`.

## Swift Overlay Future Work

- [ ] Decide in-repo vs separate-repo shape.
- [ ] Scaffold Swift app/package.
- [ ] Add `ProcessRunner` and fake process runner tests.
- [ ] Add `ForemanClient` decoding/actions tests against Rust fixtures.
- [ ] Add `OverlayStore` filtering/selection/error tests.
- [ ] Add `OverlayPanelController` AppKit boundary.
- [ ] Add global hotkey via KeyboardShortcuts.
- [ ] Add XCUITest fake-foreman harness.
- [ ] Add real tmux smoke harness.
- [ ] Add mise tasks for Swift validation.
